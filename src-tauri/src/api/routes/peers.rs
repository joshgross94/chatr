use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::PeerInfo;
use crate::services;
use crate::state::ServiceContext;

pub async fn get_room_peers(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<PeerInfo>>, (StatusCode, String)> {
    services::peers::get_room_peers(&ctx, &room_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
