use libp2p::{
    autonat, dcutr, gossipsub, identify, kad, mdns, relay, swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
pub struct ChatrBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub identify: identify::Behaviour,
    pub autonat: autonat::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub relay_client: relay::client::Behaviour,
}
