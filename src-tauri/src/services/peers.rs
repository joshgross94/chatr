use crate::models::PeerInfo;
use crate::state::ServiceContext;

pub async fn get_room_peers(
    ctx: &ServiceContext,
    room_id: &str,
) -> Result<Vec<PeerInfo>, String> {
    let peers = ctx.peers.lock().await;
    let room_peers = ctx.room_peers.lock().await;

    let peer_ids = room_peers.get(room_id).cloned().unwrap_or_default();
    let result: Vec<_> = peer_ids
        .iter()
        .filter_map(|pid| peers.get(pid).cloned())
        .collect();
    Ok(result)
}
