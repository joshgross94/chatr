use rusqlite::OptionalExtension;
use crate::models::*;
use super::Database;

impl Database {
    // ============================================================
    // Phase 0: Rooms
    // ============================================================

    pub fn create_room(&self, room: &Room) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO rooms (id, name, invite_code, created_at, owner_peer_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                room.id,
                room.name,
                room.invite_code,
                room.created_at,
                room.owner_peer_id,
            ],
        )?;
        Ok(())
    }

    pub fn list_rooms(&self) -> rusqlite::Result<Vec<Room>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, invite_code, created_at, owner_peer_id
             FROM rooms ORDER BY created_at",
        )?;
        let rooms = stmt
            .query_map([], |row| {
                Ok(Room {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    invite_code: row.get(2)?,
                    created_at: row.get(3)?,
                    owner_peer_id: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rooms)
    }

    pub fn get_room_by_invite(&self, invite_code: &str) -> rusqlite::Result<Option<Room>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, invite_code, created_at, owner_peer_id
             FROM rooms WHERE invite_code = ?1",
        )?;
        let result = stmt.query_row(rusqlite::params![invite_code], |row| {
            Ok(Room {
                id: row.get(0)?,
                name: row.get(1)?,
                invite_code: row.get(2)?,
                created_at: row.get(3)?,
                owner_peer_id: row.get(4)?,
            })
        });
        match result {
            Ok(room) => Ok(Some(room)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // ============================================================
    // Phase 0: Channels
    // ============================================================

    pub fn create_channel(&self, channel: &Channel) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO channels (id, room_id, name, created_at, channel_type, topic, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                channel.id,
                channel.room_id,
                channel.name,
                channel.created_at,
                channel.channel_type,
                channel.topic,
                channel.position,
            ],
        )?;
        Ok(())
    }

    pub fn get_room_id_for_channel(&self, channel_id: &str) -> rusqlite::Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT room_id FROM channels WHERE id = ?1")?;
        let result = stmt.query_row(rusqlite::params![channel_id], |row| row.get::<_, String>(0));
        match result {
            Ok(room_id) => Ok(Some(room_id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_channels(&self, room_id: &str) -> rusqlite::Result<Vec<Channel>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, name, created_at, channel_type, topic, position
             FROM channels WHERE room_id = ?1 ORDER BY position, created_at",
        )?;
        let channels = stmt
            .query_map(rusqlite::params![room_id], |row| {
                Ok(Channel {
                    id: row.get(0)?,
                    room_id: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                    channel_type: row.get(4)?,
                    topic: row.get(5)?,
                    position: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(channels)
    }

    // ============================================================
    // Phase 2: Channel Management
    // ============================================================

    pub fn update_channel(
        &self,
        channel_id: &str,
        name: Option<&str>,
        topic: Option<&str>,
        position: Option<i32>,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(name) = name {
            conn.execute(
                "UPDATE channels SET name = ?1 WHERE id = ?2",
                rusqlite::params![name, channel_id],
            )?;
        }
        if let Some(topic) = topic {
            conn.execute(
                "UPDATE channels SET topic = ?1 WHERE id = ?2",
                rusqlite::params![topic, channel_id],
            )?;
        }
        if let Some(position) = position {
            conn.execute(
                "UPDATE channels SET position = ?1 WHERE id = ?2",
                rusqlite::params![position, channel_id],
            )?;
        }
        Ok(())
    }

    pub fn get_channel_room_id(&self, channel_id: &str) -> rusqlite::Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT room_id FROM channels WHERE id = ?1",
            rusqlite::params![channel_id],
            |row| row.get(0),
        ).optional()
    }

    pub fn delete_channel(&self, channel_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        // Delete messages in the channel first (cascade manually for safety)
        conn.execute(
            "DELETE FROM messages WHERE channel_id = ?1",
            rusqlite::params![channel_id],
        )?;
        conn.execute(
            "DELETE FROM pinned_messages WHERE channel_id = ?1",
            rusqlite::params![channel_id],
        )?;
        conn.execute(
            "DELETE FROM channels WHERE id = ?1",
            rusqlite::params![channel_id],
        )?;
        Ok(())
    }

    // ============================================================
    // Phase 2: DM Conversations
    // ============================================================

    pub fn create_dm_conversation(&self, conv: &DmConversation) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO dm_conversations (id, is_group, name, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![conv.id, conv.is_group, conv.name, conv.created_at],
        )?;
        Ok(())
    }

    pub fn list_dm_conversations(&self) -> rusqlite::Result<Vec<DmConversation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, is_group, name, created_at
             FROM dm_conversations ORDER BY created_at DESC",
        )?;
        let convs = stmt
            .query_map([], |row| {
                Ok(DmConversation {
                    id: row.get(0)?,
                    is_group: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(convs)
    }

    pub fn get_dm_participants(
        &self,
        conversation_id: &str,
    ) -> rusqlite::Result<Vec<DmParticipant>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT conversation_id, peer_id, joined_at
             FROM dm_participants WHERE conversation_id = ?1 ORDER BY joined_at",
        )?;
        let participants = stmt
            .query_map(rusqlite::params![conversation_id], |row| {
                Ok(DmParticipant {
                    conversation_id: row.get(0)?,
                    peer_id: row.get(1)?,
                    joined_at: row.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(participants)
    }

    pub fn add_dm_participant(&self, participant: &DmParticipant) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO dm_participants (conversation_id, peer_id, joined_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![
                participant.conversation_id,
                participant.peer_id,
                participant.joined_at,
            ],
        )?;
        Ok(())
    }

    // ============================================================
    // Phase 2: Roles
    // ============================================================

    pub fn set_role(&self, role: &RoomRole) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO room_roles (id, room_id, peer_id, role, assigned_by, assigned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                role.id,
                role.room_id,
                role.peer_id,
                role.role,
                role.assigned_by,
                role.assigned_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_role(&self, room_id: &str, peer_id: &str) -> rusqlite::Result<Option<RoomRole>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, peer_id, role, assigned_by, assigned_at
             FROM room_roles WHERE room_id = ?1 AND peer_id = ?2",
        )?;
        let result = stmt.query_row(rusqlite::params![room_id, peer_id], |row| {
            Ok(RoomRole {
                id: row.get(0)?,
                room_id: row.get(1)?,
                peer_id: row.get(2)?,
                role: row.get(3)?,
                assigned_by: row.get(4)?,
                assigned_at: row.get(5)?,
            })
        });
        match result {
            Ok(role) => Ok(Some(role)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_room_roles(&self, room_id: &str) -> rusqlite::Result<Vec<RoomRole>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, peer_id, role, assigned_by, assigned_at
             FROM room_roles WHERE room_id = ?1 ORDER BY assigned_at",
        )?;
        let roles = stmt
            .query_map(rusqlite::params![room_id], |row| {
                Ok(RoomRole {
                    id: row.get(0)?,
                    room_id: row.get(1)?,
                    peer_id: row.get(2)?,
                    role: row.get(3)?,
                    assigned_by: row.get(4)?,
                    assigned_at: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(roles)
    }

    pub fn remove_role(&self, room_id: &str, peer_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM room_roles WHERE room_id = ?1 AND peer_id = ?2",
            rusqlite::params![room_id, peer_id],
        )?;
        Ok(())
    }

    // ============================================================
    // Phase 2: Moderation
    // ============================================================

    pub fn add_moderation_action(&self, action: &ModerationAction) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO moderation_actions (id, room_id, action_type, target_peer_id, moderator_peer_id, reason, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                action.id,
                action.room_id,
                action.action_type,
                action.target_peer_id,
                action.moderator_peer_id,
                action.reason,
                action.created_at,
                action.expires_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_moderation_actions(
        &self,
        room_id: &str,
    ) -> rusqlite::Result<Vec<ModerationAction>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, action_type, target_peer_id, moderator_peer_id, reason, created_at, expires_at
             FROM moderation_actions WHERE room_id = ?1 ORDER BY created_at DESC",
        )?;
        let actions = stmt
            .query_map(rusqlite::params![room_id], |row| {
                Ok(ModerationAction {
                    id: row.get(0)?,
                    room_id: row.get(1)?,
                    action_type: row.get(2)?,
                    target_peer_id: row.get(3)?,
                    moderator_peer_id: row.get(4)?,
                    reason: row.get(5)?,
                    created_at: row.get(6)?,
                    expires_at: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(actions)
    }

    pub fn block_peer(&self, peer_id: &str, blocked_at: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO blocked_peers (peer_id, blocked_at) VALUES (?1, ?2)",
            rusqlite::params![peer_id, blocked_at],
        )?;
        Ok(())
    }

    pub fn unblock_peer(&self, peer_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM blocked_peers WHERE peer_id = ?1",
            rusqlite::params![peer_id],
        )?;
        Ok(())
    }

    pub fn get_blocked_peers(&self) -> rusqlite::Result<Vec<BlockedPeer>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT peer_id, blocked_at FROM blocked_peers ORDER BY blocked_at DESC",
        )?;
        let blocked = stmt
            .query_map([], |row| {
                Ok(BlockedPeer {
                    peer_id: row.get(0)?,
                    blocked_at: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(blocked)
    }

    pub fn is_peer_banned(&self, room_id: &str, peer_id: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM moderation_actions
             WHERE room_id = ?1 AND target_peer_id = ?2 AND action_type = 'ban'
             AND (expires_at IS NULL OR expires_at > datetime('now'))",
        )?;
        let count: i64 = stmt.query_row(rusqlite::params![room_id, peer_id], |row| row.get(0))?;
        Ok(count > 0)
    }

    // ============================================================
    // Phase 4: File Sharing
    // ============================================================

    pub fn insert_file(&self, file: &FileMetadata) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO files (id, filename, size, mime_type, sha256_hash, chunk_count, uploader_peer_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                file.id,
                file.filename,
                file.size,
                file.mime_type,
                file.sha256_hash,
                file.chunk_count,
                file.uploader_peer_id,
                file.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_file(&self, file_id: &str) -> rusqlite::Result<Option<FileMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, filename, size, mime_type, sha256_hash, chunk_count, uploader_peer_id, created_at
             FROM files WHERE id = ?1",
        )?;
        let result = stmt.query_row(rusqlite::params![file_id], |row| {
            Ok(FileMetadata {
                id: row.get(0)?,
                filename: row.get(1)?,
                size: row.get(2)?,
                mime_type: row.get(3)?,
                sha256_hash: row.get(4)?,
                chunk_count: row.get(5)?,
                uploader_peer_id: row.get(6)?,
                created_at: row.get(7)?,
            })
        });
        match result {
            Ok(file) => Ok(Some(file)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn insert_message_attachment(
        &self,
        attachment: &MessageAttachment,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO message_attachments (message_id, file_id) VALUES (?1, ?2)",
            rusqlite::params![attachment.message_id, attachment.file_id],
        )?;
        Ok(())
    }

    pub fn get_message_attachments(
        &self,
        message_id: &str,
    ) -> rusqlite::Result<Vec<FileMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT f.id, f.filename, f.size, f.mime_type, f.sha256_hash, f.chunk_count, f.uploader_peer_id, f.created_at
             FROM files f
             INNER JOIN message_attachments ma ON ma.file_id = f.id
             WHERE ma.message_id = ?1",
        )?;
        let files = stmt
            .query_map(rusqlite::params![message_id], |row| {
                Ok(FileMetadata {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    size: row.get(2)?,
                    mime_type: row.get(3)?,
                    sha256_hash: row.get(4)?,
                    chunk_count: row.get(5)?,
                    uploader_peer_id: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(files)
    }

    // ============================================================
    // Phase 5: Friends
    // ============================================================

    pub fn add_friend(&self, friend: &Friend) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO friends (peer_id, display_name, status, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                friend.peer_id,
                friend.display_name,
                friend.status,
                friend.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_friend_status(&self, peer_id: &str, status: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE friends SET status = ?1 WHERE peer_id = ?2",
            rusqlite::params![status, peer_id],
        )?;
        Ok(())
    }

    pub fn remove_friend(&self, peer_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM friends WHERE peer_id = ?1",
            rusqlite::params![peer_id],
        )?;
        Ok(())
    }

    pub fn list_friends(&self) -> rusqlite::Result<Vec<Friend>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT peer_id, display_name, status, created_at
             FROM friends ORDER BY created_at DESC",
        )?;
        let friends = stmt
            .query_map([], |row| {
                Ok(Friend {
                    peer_id: row.get(0)?,
                    display_name: row.get(1)?,
                    status: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(friends)
    }

    pub fn get_friend(&self, peer_id: &str) -> rusqlite::Result<Option<Friend>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT peer_id, display_name, status, created_at
             FROM friends WHERE peer_id = ?1",
        )?;
        let result = stmt.query_row(rusqlite::params![peer_id], |row| {
            Ok(Friend {
                peer_id: row.get(0)?,
                display_name: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
            })
        });
        match result {
            Ok(friend) => Ok(Some(friend)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // ============================================================
    // Phase 6: Custom Emoji
    // ============================================================

    pub fn add_custom_emoji(&self, emoji: &CustomEmoji) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO custom_emoji (id, room_id, name, file_hash, uploaded_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                emoji.id,
                emoji.room_id,
                emoji.name,
                emoji.file_hash,
                emoji.uploaded_by,
                emoji.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn remove_custom_emoji(&self, emoji_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM custom_emoji WHERE id = ?1",
            rusqlite::params![emoji_id],
        )?;
        Ok(())
    }

    pub fn list_custom_emoji(&self, room_id: &str) -> rusqlite::Result<Vec<CustomEmoji>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, name, file_hash, uploaded_by, created_at
             FROM custom_emoji WHERE room_id = ?1 ORDER BY name",
        )?;
        let emojis = stmt
            .query_map(rusqlite::params![room_id], |row| {
                Ok(CustomEmoji {
                    id: row.get(0)?,
                    room_id: row.get(1)?,
                    name: row.get(2)?,
                    file_hash: row.get(3)?,
                    uploaded_by: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(emojis)
    }
}
