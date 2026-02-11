use axum::{routing::{delete, get, post, put}, Router};
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::api::{routes, websocket};
use crate::media::frame_server::{self, FrameServerState};
use crate::state::ServiceContext;

pub fn build_router(ctx: ServiceContext, frame_server: FrameServerState) -> Router {
    Router::new()
        // Identity
        .route("/api/v1/identity", get(routes::identity::get_identity))
        .route("/api/v1/identity/display-name", put(routes::identity::set_display_name))
        .route("/api/v1/identity/status", put(routes::identity::set_status))
        .route("/api/v1/identity/avatar", put(routes::identity::set_avatar))
        // Rooms
        .route("/api/v1/rooms", get(routes::rooms::list_rooms).post(routes::rooms::create_room))
        .route("/api/v1/rooms/join", post(routes::rooms::join_room))
        .route("/api/v1/rooms/:room_id/channels", get(routes::rooms::get_channels).post(routes::channels::create_channel))
        .route("/api/v1/rooms/:room_id/peers", get(routes::peers::get_room_peers))
        .route("/api/v1/rooms/:room_id/roles", get(routes::roles::get_room_roles).post(routes::roles::set_role))
        .route("/api/v1/rooms/:room_id/roles/:peer_id", delete(routes::roles::remove_role))
        .route("/api/v1/rooms/:room_id/moderate", post(routes::moderation::moderate))
        .route("/api/v1/rooms/:room_id/audit-log", get(routes::moderation::get_audit_log))
        .route("/api/v1/rooms/:room_id/emoji", get(routes::emoji::list_emoji).post(routes::emoji::add_emoji))
        // Channels
        .route("/api/v1/channels/:channel_id", put(routes::channels::update_channel).delete(routes::channels::delete_channel))
        .route(
            "/api/v1/channels/:channel_id/messages",
            get(routes::messaging::get_messages).post(routes::messaging::send_message),
        )
        .route("/api/v1/channels/:channel_id/typing", post(routes::messaging::typing_indicator))
        .route("/api/v1/channels/:channel_id/read", post(routes::messaging::mark_read))
        .route("/api/v1/channels/:channel_id/read-receipts", get(routes::messaging::get_read_receipts))
        .route("/api/v1/channels/:channel_id/pins", get(routes::messaging::get_pinned_messages).post(routes::messaging::pin_message))
        .route("/api/v1/channels/:channel_id/pins/:message_id", delete(routes::messaging::unpin_message))
        // Messages
        .route("/api/v1/messages/:message_id", put(routes::messaging::edit_message).delete(routes::messaging::delete_message))
        .route("/api/v1/messages/:message_id/reactions", get(routes::messaging::get_reactions).post(routes::messaging::add_reaction))
        .route("/api/v1/messages/:message_id/reactions/:emoji", delete(routes::messaging::remove_reaction))
        .route("/api/v1/messages/:message_id/attachments", get(routes::files::get_attachments).post(routes::files::attach_file))
        // Search
        .route("/api/v1/search/messages", get(routes::messaging::search_messages))
        // DMs
        .route("/api/v1/dms", get(routes::dms::list_dms).post(routes::dms::create_dm))
        .route("/api/v1/dms/:conversation_id/participants", get(routes::dms::get_dm_participants))
        .route(
            "/api/v1/dms/:conversation_id/messages",
            get(routes::dms::get_dm_messages).post(routes::dms::send_dm_message),
        )
        // Files
        .route("/api/v1/files", post(routes::files::register_file))
        .route("/api/v1/files/:file_id", get(routes::files::get_file))
        // Friends
        .route("/api/v1/friends", get(routes::friends::list_friends).post(routes::friends::send_friend_request))
        .route("/api/v1/friends/:peer_id", get(routes::friends::get_friend).delete(routes::friends::remove_friend))
        .route("/api/v1/friends/:peer_id/accept", post(routes::friends::accept_friend_request))
        // Blocked peers
        .route("/api/v1/blocked", get(routes::moderation::get_blocked_peers).post(routes::moderation::block_peer))
        .route("/api/v1/blocked/:peer_id", delete(routes::moderation::unblock_peer))
        // Emoji
        .route("/api/v1/emoji/:emoji_id", delete(routes::emoji::remove_emoji))
        // Settings
        .route("/api/v1/settings", get(routes::settings::get_all_settings))
        .route("/api/v1/settings/:key", get(routes::settings::get_setting).put(routes::settings::set_setting).delete(routes::settings::delete_setting))
        // Notifications
        .route("/api/v1/notifications", get(routes::notifications::get_all_notification_settings))
        .route("/api/v1/notifications/:target_type/:target_id", get(routes::notifications::get_notification_setting).put(routes::notifications::set_notification_setting))
        // Voice (media engine)
        .route("/api/v1/voice/join", post(routes::voice::join_voice))
        .route("/api/v1/voice/leave", post(routes::voice::leave_voice))
        .route("/api/v1/voice/muted", put(routes::voice::set_muted))
        .route("/api/v1/voice/deafened", put(routes::voice::set_deafened))
        .route("/api/v1/voice/devices", get(routes::voice::list_devices))
        .route("/api/v1/voice/state", get(routes::voice::get_voice_state))
        // Camera & Screen share
        .route("/api/v1/voice/camera/enable", post(routes::voice::enable_camera))
        .route("/api/v1/voice/camera/disable", post(routes::voice::disable_camera))
        .route("/api/v1/voice/cameras", get(routes::voice::list_cameras))
        .route("/api/v1/voice/screen/start", post(routes::voice::start_screen_share))
        .route("/api/v1/voice/screen/stop", post(routes::voice::stop_screen_share))
        // Voice signaling (legacy)
        .route("/api/v1/voice/offer", post(routes::voice::send_call_offer))
        .route("/api/v1/voice/answer", post(routes::voice::send_call_answer))
        .route("/api/v1/voice/ice-candidate", post(routes::voice::send_ice_candidate))
        .route("/api/v1/voice/broadcast-state", post(routes::voice::update_voice_state))
        // WebSocket
        .route("/ws", get(websocket::ws_handler))
        // Middleware
        .layer(CorsLayer::permissive())
        .with_state(ctx)
        // Merge frame server routes (MJPEG streams)
        .merge(frame_server::frame_server_routes(frame_server))
}

pub async fn start_api_server(ctx: ServiceContext, port: u16, frame_server: FrameServerState) {
    let router = build_router(ctx, frame_server);
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind API server");
    info!("API server listening on http://{}", addr);
    axum::serve(listener, router)
        .await
        .expect("API server error");
}
