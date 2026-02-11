use chrono::Utc;
use uuid::Uuid;

use crate::models::{FileMetadata, MessageAttachment};
use crate::state::ServiceContext;

pub fn register_file(
    ctx: &ServiceContext,
    filename: &str,
    size: i64,
    mime_type: &str,
    sha256_hash: &str,
    chunk_count: i32,
) -> Result<FileMetadata, String> {
    let file = FileMetadata {
        id: Uuid::new_v4().to_string(),
        filename: filename.to_string(),
        size,
        mime_type: mime_type.to_string(),
        sha256_hash: sha256_hash.to_string(),
        chunk_count,
        uploader_peer_id: ctx.peer_id.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    ctx.db.insert_file(&file).map_err(|e| e.to_string())?;
    Ok(file)
}

pub fn get_file(ctx: &ServiceContext, file_id: &str) -> Result<Option<FileMetadata>, String> {
    ctx.db.get_file(file_id).map_err(|e| e.to_string())
}

pub fn attach_file(ctx: &ServiceContext, message_id: &str, file_id: &str) -> Result<(), String> {
    let attachment = MessageAttachment {
        message_id: message_id.to_string(),
        file_id: file_id.to_string(),
    };
    ctx.db.insert_message_attachment(&attachment).map_err(|e| e.to_string())
}

pub fn get_attachments(ctx: &ServiceContext, message_id: &str) -> Result<Vec<FileMetadata>, String> {
    ctx.db.get_message_attachments(message_id).map_err(|e| e.to_string())
}
