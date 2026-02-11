use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use libp2p::{
    autonat, dcutr, gossipsub, identify, kad,
    mdns, noise, relay, tcp, yamux,
    Multiaddr, PeerId, Swarm, SwarmBuilder,
    swarm::SwarmEvent,
};
use libp2p::futures::StreamExt;
use libp2p::identity::Keypair;
use tokio::sync::{mpsc, Mutex as TokioMutex};
use tracing::{info, warn, debug, error};

use crate::db::Database;
use crate::events::{AppEvent, EventSender};
use crate::models::{ChatMessage, NetworkMessage, PeerInfo, CallOfferNet, CallAnswerNet, IceCandidateNet, VoiceStateNet, ChannelCreatedNet, ChannelDeletedNet, ChannelSyncNet};
use crate::network::behaviour::{ChatrBehaviour, ChatrBehaviourEvent};
use crate::network::bootstrap;
use crate::network::NetworkCommand;

const PROTOCOL_VERSION: &str = "chatr/0.1.0";

pub fn build_swarm(keypair: &Keypair) -> Result<Swarm<ChatrBehaviour>, Box<dyn std::error::Error>> {
    let peer_id = PeerId::from(keypair.public());

    // GossipSub config
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .mesh_n(2)
        .mesh_n_low(1)
        .mesh_n_high(4)
        .mesh_outbound_min(1)
        .flood_publish(true)
        .build()
        .map_err(|e| format!("GossipSub config error: {}", e))?;

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )
    .map_err(|e| format!("GossipSub behaviour error: {}", e))?;

    // mDNS for LAN discovery
    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

    // Kademlia for DHT
    let kademlia_config = kad::Config::new(libp2p::StreamProtocol::new("/chatr/kad/1.0.0"));
    let store = kad::store::MemoryStore::new(peer_id);
    let mut kademlia = kad::Behaviour::with_config(peer_id, store, kademlia_config);

    // Add IPFS bootstrap nodes to Kademlia
    for addr in bootstrap::bootstrap_nodes() {
        if let Some(bootstrap_peer_id) = addr.iter().find_map(|p| {
            if let libp2p::multiaddr::Protocol::P2p(id) = p {
                Some(id)
            } else {
                None
            }
        }) {
            kademlia.add_address(&bootstrap_peer_id, addr.clone());
        }
    }

    // Identify protocol
    let identify = identify::Behaviour::new(identify::Config::new(
        PROTOCOL_VERSION.to_string(),
        keypair.public(),
    ));

    // AutoNAT
    let autonat = autonat::Behaviour::new(peer_id, autonat::Config::default());

    // DCUtR for hole punching
    let dcutr = dcutr::Behaviour::new(peer_id);

    let swarm = SwarmBuilder::with_existing_identity(keypair.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|_key, relay_client| {
            Ok(ChatrBehaviour {
                gossipsub,
                mdns,
                kademlia,
                identify,
                autonat,
                dcutr,
                relay_client,
            })
        })?
        .with_swarm_config(|c: libp2p::swarm::Config| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Ok(swarm)
}

pub async fn run_event_loop(
    mut swarm: Swarm<ChatrBehaviour>,
    mut cmd_rx: mpsc::Receiver<NetworkCommand>,
    db: Arc<Database>,
    event_tx: EventSender,
    my_peer_id: String,
    peers: Arc<TokioMutex<HashMap<String, PeerInfo>>>,
    room_peers: Arc<TokioMutex<HashMap<String, HashSet<String>>>>,
) {
    // Listen on all interfaces
    let listen_addr_tcp: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
    let listen_addr_quic: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap();

    swarm.listen_on(listen_addr_tcp).expect("Failed to listen on TCP");
    swarm.listen_on(listen_addr_quic).expect("Failed to listen on QUIC");

    // Bootstrap Kademlia
    if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
        warn!("Kademlia bootstrap failed (expected if no peers yet): {}", e);
    }

    // Subscribe to the global discovery topic for room lookups
    let discovery_topic = gossipsub::IdentTopic::new(crate::network::DISCOVERY_TOPIC);
    if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&discovery_topic) {
        warn!("Failed to subscribe to discovery topic: {}", e);
    } else {
        info!("Subscribed to discovery topic");
    }

    // Track subscribed room channel topics
    let mut subscribed_topics: HashSet<String> = HashSet::new();
    // Track known peer display names (from PeerAnnounce messages)
    let mut peer_names: HashMap<String, String> = HashMap::new();
    // Pending DHT lookups
    let mut pending_dht_lookups: HashMap<kad::QueryId, tokio::sync::oneshot::Sender<Option<(String, String)>>> = HashMap::new();
    // Pending GossipSub room lookups: invite_code -> oneshot sender
    let mut pending_gossip_lookups: HashMap<String, tokio::sync::oneshot::Sender<Option<(String, String)>>> = HashMap::new();

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                        for (peer_id, addr) in peers {
                            info!("mDNS discovered peer: {} at {}", peer_id, addr);
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                        }
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                        for (peer_id, _addr) in peers {
                            info!("mDNS peer expired: {}", peer_id);
                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        message,
                        propagation_source,
                        ..
                    })) => {
                        debug!("GossipSub message from {}", propagation_source);
                        if let Ok(net_msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                            match net_msg {
                                NetworkMessage::Chat(chat_msg) => {
                                    info!("Received chat message from {} in channel {}: {}", chat_msg.sender_display_name, chat_msg.channel_id, chat_msg.content);
                                    if chat_msg.sender_peer_id != my_peer_id {
                                        let msg = crate::models::Message {
                                            id: chat_msg.id.clone(),
                                            channel_id: chat_msg.channel_id.clone(),
                                            sender_peer_id: chat_msg.sender_peer_id.clone(),
                                            sender_display_name: chat_msg.sender_display_name.clone(),
                                            content: chat_msg.content.clone(),
                                            timestamp: chat_msg.timestamp.clone(),
                                            edited_at: None,
                                            deleted_at: None,
                                            reply_to_id: chat_msg.reply_to_id.clone(),
                                        };
                                        if let Err(e) = db.insert_message(&msg) {
                                            error!("Failed to insert message: {}", e);
                                        }
                                        let _ = event_tx.send(AppEvent::NewMessage(msg));
                                    }
                                }
                                NetworkMessage::PeerAnnounce(announce) => {
                                    info!("Peer announced: {} ({})", announce.display_name, announce.peer_id);
                                    peer_names.insert(announce.peer_id.clone(), announce.display_name.clone());
                                    let peer_info = PeerInfo {
                                        peer_id: announce.peer_id.clone(),
                                        display_name: announce.display_name.clone(),
                                        is_online: true,
                                    };
                                    // Update shared peers map so API/services see correct names
                                    {
                                        let mut p = peers.lock().await;
                                        p.insert(announce.peer_id.clone(), peer_info.clone());
                                    }
                                    // Track peer in room
                                    {
                                        let mut rp = room_peers.lock().await;
                                        rp.entry(announce.room_id.clone())
                                            .or_default()
                                            .insert(announce.peer_id.clone());
                                    }
                                    let _ = event_tx.send(AppEvent::PeerDiscovered(peer_info));
                                }
                                NetworkMessage::RoomLookup(req) => {
                                    // Someone is looking for a room by invite code - check if we have it
                                    if req.requester_peer_id != my_peer_id {
                                        info!("Received room lookup for invite code: {}", req.invite_code);
                                        if let Ok(Some(room)) = db.get_room_by_invite(&req.invite_code) {
                                            // We have this room, respond on the discovery topic
                                            let response = NetworkMessage::RoomFound(crate::models::RoomLookupResponse {
                                                invite_code: req.invite_code,
                                                room_id: room.id,
                                                room_name: room.name,
                                                target_peer_id: req.requester_peer_id,
                                            });
                                            if let Ok(data) = serde_json::to_vec(&response) {
                                                let disc_topic = gossipsub::IdentTopic::new(crate::network::DISCOVERY_TOPIC);
                                                let _ = swarm.behaviour_mut().gossipsub.publish(disc_topic, data);
                                            }
                                        }
                                    }
                                }
                                NetworkMessage::RoomFound(resp) => {
                                    // Someone responded with room info - check if it's for us
                                    if resp.target_peer_id == my_peer_id {
                                        info!("Received room info for invite {}: {} ({})", resp.invite_code, resp.room_name, resp.room_id);
                                        if let Some(sender) = pending_gossip_lookups.remove(&resp.invite_code) {
                                            let _ = sender.send(Some((resp.room_id, resp.room_name)));
                                        }
                                    }
                                }
                                NetworkMessage::MessageEdit(edit) => {
                                    if edit.sender_peer_id != my_peer_id {
                                        info!("Received message edit from {}: {}", edit.sender_peer_id, edit.message_id);
                                        let _ = db.edit_message(&edit.message_id, &edit.new_content, &edit.edited_at);
                                        let _ = event_tx.send(AppEvent::MessageEdited {
                                            message_id: edit.message_id,
                                            channel_id: edit.channel_id,
                                            new_content: edit.new_content,
                                            edited_at: edit.edited_at,
                                        });
                                    }
                                }
                                NetworkMessage::MessageDelete(del) => {
                                    if del.sender_peer_id != my_peer_id {
                                        info!("Received message delete from {}: {}", del.sender_peer_id, del.message_id);
                                        let _ = db.delete_message(&del.message_id, &del.deleted_at);
                                        let _ = event_tx.send(AppEvent::MessageDeleted {
                                            message_id: del.message_id,
                                            channel_id: del.channel_id,
                                        });
                                    }
                                }
                                NetworkMessage::Reaction(reaction) => {
                                    if reaction.peer_id != my_peer_id {
                                        if reaction.add {
                                            let r = crate::models::Reaction {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                message_id: reaction.message_id.clone(),
                                                peer_id: reaction.peer_id.clone(),
                                                emoji: reaction.emoji.clone(),
                                                created_at: chrono::Utc::now().to_rfc3339(),
                                            };
                                            let _ = db.add_reaction(&r);
                                            let _ = event_tx.send(AppEvent::ReactionAdded {
                                                message_id: reaction.message_id,
                                                channel_id: reaction.channel_id,
                                                peer_id: reaction.peer_id,
                                                emoji: reaction.emoji,
                                            });
                                        } else {
                                            let _ = db.remove_reaction(&reaction.message_id, &reaction.peer_id, &reaction.emoji);
                                            let _ = event_tx.send(AppEvent::ReactionRemoved {
                                                message_id: reaction.message_id,
                                                channel_id: reaction.channel_id,
                                                peer_id: reaction.peer_id,
                                                emoji: reaction.emoji,
                                            });
                                        }
                                    }
                                }
                                NetworkMessage::TypingIndicator(ti) => {
                                    if ti.peer_id != my_peer_id {
                                        if ti.typing {
                                            let _ = event_tx.send(AppEvent::TypingStarted {
                                                channel_id: ti.channel_id,
                                                peer_id: ti.peer_id,
                                                display_name: ti.display_name,
                                            });
                                        } else {
                                            let _ = event_tx.send(AppEvent::TypingStopped {
                                                channel_id: ti.channel_id,
                                                peer_id: ti.peer_id,
                                            });
                                        }
                                    }
                                }
                                NetworkMessage::ReadReceipt(rr) => {
                                    if rr.peer_id != my_peer_id {
                                        let _ = db.set_read_receipt(&rr.channel_id, &rr.peer_id, &rr.last_read_message_id, &chrono::Utc::now().to_rfc3339());
                                        let _ = event_tx.send(AppEvent::ReadReceiptUpdated {
                                            channel_id: rr.channel_id,
                                            peer_id: rr.peer_id,
                                            last_read_message_id: rr.last_read_message_id,
                                        });
                                    }
                                }
                                NetworkMessage::DmMessage(dm) => {
                                    if dm.sender_peer_id != my_peer_id {
                                        let _ = db.insert_dm_message(&dm.id, &dm.conversation_id, &dm.sender_peer_id, &dm.sender_display_name, &dm.content, &dm.timestamp);
                                        let msg = crate::models::DmMessage {
                                            id: dm.id,
                                            conversation_id: dm.conversation_id,
                                            sender_peer_id: dm.sender_peer_id,
                                            sender_display_name: dm.sender_display_name,
                                            content: dm.content,
                                            timestamp: dm.timestamp,
                                        };
                                        let _ = event_tx.send(AppEvent::NewDmMessage(msg));
                                    }
                                }
                                NetworkMessage::FriendRequest(fr) => {
                                    if fr.to_peer_id == my_peer_id {
                                        match fr.action.as_str() {
                                            "request" => {
                                                let friend = crate::models::Friend {
                                                    peer_id: fr.from_peer_id.clone(),
                                                    display_name: fr.from_display_name.clone(),
                                                    status: "pending_incoming".to_string(),
                                                    created_at: chrono::Utc::now().to_rfc3339(),
                                                };
                                                let _ = db.add_friend(&friend);
                                                let _ = event_tx.send(AppEvent::FriendRequestReceived {
                                                    from_peer_id: fr.from_peer_id,
                                                    from_display_name: fr.from_display_name,
                                                });
                                            }
                                            "accept" => {
                                                let _ = db.update_friend_status(&fr.from_peer_id, "accepted");
                                                let _ = event_tx.send(AppEvent::FriendRequestAccepted {
                                                    peer_id: fr.from_peer_id,
                                                });
                                            }
                                            "remove" => {
                                                let _ = db.remove_friend(&fr.from_peer_id);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                NetworkMessage::CallOffer(offer) => {
                                    if offer.to_peer_id == my_peer_id {
                                        info!("Received call offer from {}", offer.from_peer_id);
                                        let _ = event_tx.send(AppEvent::CallOfferReceived {
                                            call_id: offer.call_id,
                                            from_peer_id: offer.from_peer_id,
                                            channel_id: offer.channel_id,
                                            sdp: offer.sdp,
                                        });
                                    }
                                }
                                NetworkMessage::CallAnswer(answer) => {
                                    if answer.to_peer_id == my_peer_id {
                                        info!("Received call answer from {}", answer.from_peer_id);
                                        let _ = event_tx.send(AppEvent::CallAnswerReceived {
                                            call_id: answer.call_id,
                                            from_peer_id: answer.from_peer_id,
                                            channel_id: answer.channel_id,
                                            sdp: answer.sdp,
                                        });
                                    }
                                }
                                NetworkMessage::IceCandidate(ice) => {
                                    if ice.to_peer_id == my_peer_id {
                                        debug!("Received ICE candidate from {}", ice.from_peer_id);
                                        let _ = event_tx.send(AppEvent::IceCandidateReceived {
                                            from_peer_id: ice.from_peer_id,
                                            channel_id: ice.channel_id,
                                            candidate: ice.candidate,
                                        });
                                    }
                                }
                                NetworkMessage::VoiceState(vs) => {
                                    if vs.peer_id != my_peer_id {
                                        info!("Voice state from {}: channel={:?}", vs.peer_id, vs.channel_id);
                                        let _ = event_tx.send(AppEvent::VoiceStateChanged {
                                            peer_id: vs.peer_id,
                                            display_name: vs.display_name,
                                            channel_id: vs.channel_id,
                                            room_id: vs.room_id,
                                            muted: vs.muted,
                                            deafened: vs.deafened,
                                            video: vs.video,
                                            screen_sharing: vs.screen_sharing,
                                        });
                                    }
                                }
                                NetworkMessage::ChannelCreated(ch) => {
                                    info!("Received channel created: {} in room {}", ch.name, ch.room_id);
                                    // Save to local DB if we're in this room
                                    let channel = crate::models::Channel {
                                        id: ch.channel_id.clone(),
                                        room_id: ch.room_id.clone(),
                                        name: ch.name.clone(),
                                        created_at: ch.created_at.clone(),
                                        channel_type: ch.channel_type.clone(),
                                        topic: None,
                                        position: 0,
                                    };
                                    let _ = db.create_channel(&channel);
                                    let _ = event_tx.send(AppEvent::ChannelCreated {
                                        room_id: ch.room_id,
                                        channel_id: ch.channel_id,
                                        name: ch.name,
                                        channel_type: ch.channel_type,
                                        created_at: ch.created_at,
                                    });
                                }
                                NetworkMessage::ChannelDeleted(ch) => {
                                    info!("Received channel deleted: {} in room {}", ch.channel_id, ch.room_id);
                                    let _ = db.delete_channel(&ch.channel_id);
                                    let _ = event_tx.send(AppEvent::ChannelDeleted {
                                        room_id: ch.room_id,
                                        channel_id: ch.channel_id,
                                    });
                                }
                                NetworkMessage::ChannelSync { room_id, channels } => {
                                    info!("Received channel sync for room {} with {} channels", room_id, channels.len());
                                    for ch in channels {
                                        // Only insert if we don't already have this channel
                                        if db.get_channels(&room_id).map(|chs| !chs.iter().any(|c| c.id == ch.channel_id)).unwrap_or(false) {
                                            let channel = crate::models::Channel {
                                                id: ch.channel_id.clone(),
                                                room_id: room_id.clone(),
                                                name: ch.name.clone(),
                                                created_at: ch.created_at.clone(),
                                                channel_type: ch.channel_type.clone(),
                                                topic: ch.topic.clone(),
                                                position: ch.position,
                                            };
                                            let _ = db.create_channel(&channel);
                                            let _ = event_tx.send(AppEvent::ChannelCreated {
                                                room_id: room_id.clone(),
                                                channel_id: ch.channel_id,
                                                name: ch.name,
                                                channel_type: ch.channel_type,
                                                created_at: ch.created_at,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed {
                        peer_id,
                        topic,
                    })) => {
                        info!("Peer {} subscribed to {}", peer_id, topic);
                        let topic_str = topic.to_string();
                        if let Some(room_id) = topic_str.strip_prefix("chatr/room/") {
                            let room_id = room_id.split('/').next().unwrap_or(room_id);
                            let pid = peer_id.to_string();
                            let name = peer_names.get(&pid).cloned().unwrap_or_else(|| pid.chars().take(8).collect());
                            let peer_info = PeerInfo {
                                peer_id: pid.clone(),
                                display_name: name,
                                is_online: true,
                            };
                            // Update shared room_peers map
                            {
                                let mut rp = room_peers.lock().await;
                                rp.entry(room_id.to_string()).or_default().insert(pid.clone());
                            }
                            // Update shared peers map (may already exist from PeerAnnounce)
                            {
                                let mut p = peers.lock().await;
                                p.entry(pid.clone()).or_insert_with(|| peer_info.clone());
                            }
                            let _ = event_tx.send(AppEvent::PeerJoinedRoom {
                                room_id: room_id.to_string(),
                                peer: peer_info,
                            });

                            // Re-announce our presence so the new peer learns our display name
                            if subscribed_topics.contains(&topic_str) {
                                let display_name = db.get_display_name().unwrap_or_else(|_| "Anonymous".to_string());
                                let net_msg = NetworkMessage::PeerAnnounce(crate::models::PeerAnnouncement {
                                    peer_id: my_peer_id.clone(),
                                    display_name,
                                    room_id: room_id.to_string(),
                                });
                                if let Ok(data) = serde_json::to_vec(&net_msg) {
                                    let announce_topic = gossipsub::IdentTopic::new(&topic_str);
                                    let _ = swarm.behaviour_mut().gossipsub.publish(announce_topic, data);
                                }

                                // Also send channel sync so new peer gets all channels
                                if let Ok(channels) = db.get_channels(room_id) {
                                    let channel_list: Vec<ChannelSyncNet> = channels.into_iter().map(|ch| ChannelSyncNet {
                                        channel_id: ch.id,
                                        name: ch.name,
                                        channel_type: ch.channel_type,
                                        created_at: ch.created_at,
                                        topic: ch.topic,
                                        position: ch.position,
                                    }).collect();
                                    if !channel_list.is_empty() {
                                        let sync_msg = NetworkMessage::ChannelSync {
                                            room_id: room_id.to_string(),
                                            channels: channel_list,
                                        };
                                        if let Ok(data) = serde_json::to_vec(&sync_msg) {
                                            let sync_topic = gossipsub::IdentTopic::new(&topic_str);
                                            let _ = swarm.behaviour_mut().gossipsub.publish(sync_topic, data);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Gossipsub(gossipsub::Event::Unsubscribed {
                        peer_id,
                        topic,
                    })) => {
                        info!("Peer {} unsubscribed from {}", peer_id, topic);
                        let topic_str = topic.to_string();
                        if let Some(room_id) = topic_str.strip_prefix("chatr/room/") {
                            let room_id = room_id.split('/').next().unwrap_or(room_id);
                            let pid = peer_id.to_string();
                            // Remove from shared room_peers map
                            {
                                let mut rp = room_peers.lock().await;
                                if let Some(set) = rp.get_mut(room_id) {
                                    set.remove(&pid);
                                }
                            }
                            let _ = event_tx.send(AppEvent::PeerLeftRoom {
                                room_id: room_id.to_string(),
                                peer_id: pid,
                            });
                        }
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
                        id,
                        result: kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                            record,
                            ..
                        }))),
                        ..
                    })) => {
                        if let Some(sender) = pending_dht_lookups.remove(&id) {
                            if let Ok(value) = String::from_utf8(record.value) {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&value) {
                                    let room_id = parsed["room_id"].as_str().unwrap_or_default().to_string();
                                    let room_name = parsed["room_name"].as_str().unwrap_or_default().to_string();
                                    let _ = sender.send(Some((room_id, room_name)));
                                } else {
                                    let _ = sender.send(None);
                                }
                            } else {
                                let _ = sender.send(None);
                            }
                        }
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
                        id,
                        result: kad::QueryResult::GetRecord(Err(_)),
                        ..
                    })) => {
                        if let Some(sender) = pending_dht_lookups.remove(&id) {
                            let _ = sender.send(None);
                        }
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Identify(identify::Event::Received {
                        peer_id,
                        info,
                        ..
                    })) => {
                        info!("Identified peer: {} running {}", peer_id, info.protocol_version);
                        for addr in info.listen_addrs {
                            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                        }
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Autonat(autonat::Event::StatusChanged { old, new })) => {
                        info!("AutoNAT status changed: {:?} -> {:?}", old, new);
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::RelayClient(relay::client::Event::ReservationReqAccepted { relay_peer_id, .. })) => {
                        info!("Relay reservation accepted by {}", relay_peer_id);
                    }
                    SwarmEvent::Behaviour(ChatrBehaviourEvent::Dcutr(dcutr::Event { remote_peer_id, result })) => {
                        match result {
                            Ok(_) => info!("DCUtR hole punch succeeded with {}", remote_peer_id),
                            Err(e) => warn!("DCUtR hole punch failed with {}: {}", remote_peer_id, e),
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Connected to {}", peer_id);
                        let pid = peer_id.to_string();
                        let name = peer_names.get(&pid).cloned().unwrap_or_else(|| pid.chars().take(8).collect());
                        let peer_info = PeerInfo {
                            peer_id: pid.clone(),
                            display_name: name,
                            is_online: true,
                        };
                        // Update shared peers map
                        {
                            let mut p = peers.lock().await;
                            p.entry(pid).or_insert_with(|| peer_info.clone());
                        }
                        let _ = event_tx.send(AppEvent::PeerConnected(peer_info));
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        info!("Disconnected from {}", peer_id);
                        let pid = peer_id.to_string();
                        // Mark peer as offline in shared map
                        {
                            let mut p = peers.lock().await;
                            if let Some(info) = p.get_mut(&pid) {
                                info.is_online = false;
                            }
                        }
                        let _ = event_tx.send(AppEvent::PeerDisconnected {
                            peer_id: pid,
                        });
                    }
                    _ => {}
                }
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    NetworkCommand::SendMessage { room_id, message } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        let topic = gossipsub::IdentTopic::new(&topic_str);
                        let net_msg = NetworkMessage::Chat(ChatMessage {
                            id: message.id,
                            channel_id: message.channel_id,
                            sender_peer_id: message.sender_peer_id,
                            sender_display_name: message.sender_display_name,
                            content: message.content.clone(),
                            timestamp: message.timestamp,
                            reply_to_id: message.reply_to_id,
                            attachments: None,
                        });
                        if let Ok(data) = serde_json::to_vec(&net_msg) {
                            match swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                Ok(msg_id) => info!("Published message to {}: {:?} content={}", topic_str, msg_id, message.content),
                                Err(e) => warn!("Failed to publish message to {}: {}", topic_str, e),
                            }
                        }
                    }
                    NetworkCommand::SubscribeRoom { room_id } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        if !subscribed_topics.contains(&topic_str) {
                            let topic = gossipsub::IdentTopic::new(&topic_str);
                            if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                                warn!("Failed to subscribe to {}: {}", topic_str, e);
                            } else {
                                subscribed_topics.insert(topic_str.clone());
                                info!("Subscribed to room topic: chatr/room/{}", room_id);

                                // Auto-announce presence with display name
                                let display_name = db.get_display_name().unwrap_or_else(|_| "Anonymous".to_string());
                                let net_msg = NetworkMessage::PeerAnnounce(crate::models::PeerAnnouncement {
                                    peer_id: my_peer_id.clone(),
                                    display_name,
                                    room_id: room_id.clone(),
                                });
                                if let Ok(data) = serde_json::to_vec(&net_msg) {
                                    let announce_topic = gossipsub::IdentTopic::new(&topic_str);
                                    let _ = swarm.behaviour_mut().gossipsub.publish(announce_topic, data);
                                }
                            }
                        }
                    }
                    NetworkCommand::PublishRoomToDHT { room_id, invite_code, room_name } => {
                        let key = kad::RecordKey::new(&format!("chatr/invite/{}", invite_code));
                        let value = serde_json::json!({
                            "room_id": room_id,
                            "room_name": room_name,
                            "invite_code": invite_code,
                        });
                        let record = kad::Record {
                            key,
                            value: serde_json::to_vec(&value).unwrap_or_default(),
                            publisher: None,
                            expires: None,
                        };
                        if let Err(e) = swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One) {
                            warn!("Failed to publish room to DHT: {}", e);
                        } else {
                            info!("Published room {} to DHT with invite {}", room_id, invite_code);
                        }
                    }
                    NetworkCommand::LookupRoomInDHT { invite_code, reply } => {
                        let key = kad::RecordKey::new(&format!("chatr/invite/{}", invite_code));
                        let query_id = swarm.behaviour_mut().kademlia.get_record(key);
                        pending_dht_lookups.insert(query_id, reply);
                    }
                    NetworkCommand::LookupRoomViaGossip { invite_code, reply } => {
                        // Broadcast a room lookup request on the discovery topic
                        let req = NetworkMessage::RoomLookup(crate::models::RoomLookupRequest {
                            invite_code: invite_code.clone(),
                            requester_peer_id: my_peer_id.clone(),
                        });
                        if let Ok(data) = serde_json::to_vec(&req) {
                            let disc_topic = gossipsub::IdentTopic::new(crate::network::DISCOVERY_TOPIC);
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(disc_topic, data) {
                                warn!("Failed to publish room lookup: {}", e);
                                let _ = reply.send(None);
                            } else {
                                info!("Published room lookup for invite code: {}", invite_code);
                                pending_gossip_lookups.insert(invite_code, reply);
                            }
                        } else {
                            let _ = reply.send(None);
                        }
                    }
                    NetworkCommand::AnnouncePresence { room_id, display_name } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        let topic = gossipsub::IdentTopic::new(&topic_str);
                        let net_msg = NetworkMessage::PeerAnnounce(crate::models::PeerAnnouncement {
                            peer_id: my_peer_id.clone(),
                            display_name,
                            room_id,
                        });
                        if let Ok(data) = serde_json::to_vec(&net_msg) {
                            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                        }
                    }
                    NetworkCommand::SendCallOffer { room_id, to_peer_id, call_id, channel_id, sdp } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        let topic = gossipsub::IdentTopic::new(&topic_str);
                        let net_msg = NetworkMessage::CallOffer(CallOfferNet {
                            call_id,
                            from_peer_id: my_peer_id.clone(),
                            to_peer_id,
                            channel_id,
                            sdp,
                        });
                        if let Ok(data) = serde_json::to_vec(&net_msg) {
                            match swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                Ok(_) => info!("Sent call offer on {}", topic_str),
                                Err(e) => warn!("Failed to send call offer: {}", e),
                            }
                        }
                    }
                    NetworkCommand::SendCallAnswer { room_id, to_peer_id, call_id, channel_id, sdp } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        let topic = gossipsub::IdentTopic::new(&topic_str);
                        let net_msg = NetworkMessage::CallAnswer(CallAnswerNet {
                            call_id,
                            from_peer_id: my_peer_id.clone(),
                            to_peer_id,
                            channel_id,
                            sdp,
                        });
                        if let Ok(data) = serde_json::to_vec(&net_msg) {
                            match swarm.behaviour_mut().gossipsub.publish(topic, data) {
                                Ok(_) => info!("Sent call answer on {}", topic_str),
                                Err(e) => warn!("Failed to send call answer: {}", e),
                            }
                        }
                    }
                    NetworkCommand::SendIceCandidate { room_id, to_peer_id, channel_id, candidate } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        let topic = gossipsub::IdentTopic::new(&topic_str);
                        let net_msg = NetworkMessage::IceCandidate(IceCandidateNet {
                            from_peer_id: my_peer_id.clone(),
                            to_peer_id,
                            channel_id,
                            candidate,
                        });
                        if let Ok(data) = serde_json::to_vec(&net_msg) {
                            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                        }
                    }
                    NetworkCommand::SendVoiceState { room_id, channel_id, muted, deafened, video, screen_sharing } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        let topic = gossipsub::IdentTopic::new(&topic_str);
                        let display_name = db.get_display_name().unwrap_or_else(|_| "Anonymous".to_string());
                        let net_msg = NetworkMessage::VoiceState(VoiceStateNet {
                            peer_id: my_peer_id.clone(),
                            display_name,
                            channel_id,
                            room_id,
                            muted,
                            deafened,
                            video,
                            screen_sharing,
                        });
                        if let Ok(data) = serde_json::to_vec(&net_msg) {
                            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                        }
                    }
                    NetworkCommand::BroadcastChannelCreated { room_id, channel_id, name, channel_type, created_at } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        let topic = gossipsub::IdentTopic::new(&topic_str);
                        let net_msg = NetworkMessage::ChannelCreated(ChannelCreatedNet {
                            room_id,
                            channel_id,
                            name,
                            channel_type,
                            created_at,
                        });
                        if let Ok(data) = serde_json::to_vec(&net_msg) {
                            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                        }
                    }
                    NetworkCommand::BroadcastChannelDeleted { room_id, channel_id } => {
                        let topic_str = format!("chatr/room/{}", room_id);
                        let topic = gossipsub::IdentTopic::new(&topic_str);
                        let net_msg = NetworkMessage::ChannelDeleted(ChannelDeletedNet {
                            room_id,
                            channel_id,
                        });
                        if let Ok(data) = serde_json::to_vec(&net_msg) {
                            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                        }
                    }
                }
            }
        }
    }
}
