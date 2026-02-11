use crate::models::*;
use super::Database;

impl Database {
    // ============================================================
    // Phase 0: Core Message Operations
    // ============================================================

    pub fn insert_message(&self, msg: &Message) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO messages (id, channel_id, sender_peer_id, sender_display_name, content, timestamp, edited_at, deleted_at, reply_to_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                msg.id,
                msg.channel_id,
                msg.sender_peer_id,
                msg.sender_display_name,
                msg.content,
                msg.timestamp,
                msg.edited_at,
                msg.deleted_at,
                msg.reply_to_id,
            ],
        )?;
        Ok(())
    }

    pub fn get_messages(&self, channel_id: &str, limit: i64, before: Option<&str>) -> rusqlite::Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut messages = if let Some(before_ts) = before {
            let mut stmt = conn.prepare(
                "SELECT id, channel_id, sender_peer_id, sender_display_name, content, timestamp, edited_at, deleted_at, reply_to_id
                 FROM messages
                 WHERE channel_id = ?1 AND timestamp < ?2 AND deleted_at IS NULL
                 ORDER BY timestamp DESC LIMIT ?3",
            )?;
            let rows = stmt.query_map(rusqlite::params![channel_id, before_ts, limit], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    sender_peer_id: row.get(2)?,
                    sender_display_name: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                    edited_at: row.get(6)?,
                    deleted_at: row.get(7)?,
                    reply_to_id: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, channel_id, sender_peer_id, sender_display_name, content, timestamp, edited_at, deleted_at, reply_to_id
                 FROM messages
                 WHERE channel_id = ?1 AND deleted_at IS NULL
                 ORDER BY timestamp DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![channel_id, limit], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    sender_peer_id: row.get(2)?,
                    sender_display_name: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                    edited_at: row.get(6)?,
                    deleted_at: row.get(7)?,
                    reply_to_id: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        };
        messages.reverse();
        Ok(messages)
    }

    // ============================================================
    // Phase 1: Edit, Delete, Reactions, Read Receipts, Search
    // ============================================================

    pub fn edit_message(&self, message_id: &str, new_content: &str, edited_at: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE messages SET content = ?1, edited_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
            rusqlite::params![new_content, edited_at, message_id],
        )?;
        Ok(rows_affected > 0)
    }

    pub fn delete_message(&self, message_id: &str, deleted_at: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE messages SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            rusqlite::params![deleted_at, message_id],
        )?;
        Ok(rows_affected > 0)
    }

    pub fn add_reaction(&self, reaction: &Reaction) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO reactions (id, message_id, peer_id, emoji, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                reaction.id,
                reaction.message_id,
                reaction.peer_id,
                reaction.emoji,
                reaction.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn remove_reaction(&self, message_id: &str, peer_id: &str, emoji: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "DELETE FROM reactions WHERE message_id = ?1 AND peer_id = ?2 AND emoji = ?3",
            rusqlite::params![message_id, peer_id, emoji],
        )?;
        Ok(rows_affected > 0)
    }

    pub fn get_reactions(&self, message_id: &str) -> rusqlite::Result<Vec<Reaction>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, message_id, peer_id, emoji, created_at
             FROM reactions
             WHERE message_id = ?1
             ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![message_id], |row| {
            Ok(Reaction {
                id: row.get(0)?,
                message_id: row.get(1)?,
                peer_id: row.get(2)?,
                emoji: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn set_read_receipt(&self, channel_id: &str, peer_id: &str, last_read_message_id: &str, updated_at: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO read_receipts (channel_id, peer_id, last_read_message_id, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![channel_id, peer_id, last_read_message_id, updated_at],
        )?;
        Ok(())
    }

    pub fn get_read_receipts(&self, channel_id: &str) -> rusqlite::Result<Vec<ReadReceipt>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT channel_id, peer_id, last_read_message_id, updated_at
             FROM read_receipts
             WHERE channel_id = ?1
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![channel_id], |row| {
            Ok(ReadReceipt {
                channel_id: row.get(0)?,
                peer_id: row.get(1)?,
                last_read_message_id: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn search_messages(&self, channel_id: Option<&str>, query: &str, limit: i64, offset: i64) -> rusqlite::Result<SearchResult> {
        let conn = self.conn.lock().unwrap();

        let (total, messages) = if let Some(ch_id) = channel_id {
            let total: i64 = conn.query_row(
                "SELECT COUNT(*)
                 FROM messages_fts
                 JOIN messages ON messages.rowid = messages_fts.rowid
                 WHERE messages_fts MATCH ?1 AND messages_fts.channel_id = ?2 AND messages.deleted_at IS NULL",
                rusqlite::params![query, ch_id],
                |row| row.get(0),
            )?;

            let mut stmt = conn.prepare(
                "SELECT messages.id, messages.channel_id, messages.sender_peer_id, messages.sender_display_name,
                        messages.content, messages.timestamp, messages.edited_at, messages.deleted_at, messages.reply_to_id
                 FROM messages_fts
                 JOIN messages ON messages.rowid = messages_fts.rowid
                 WHERE messages_fts MATCH ?1 AND messages_fts.channel_id = ?2 AND messages.deleted_at IS NULL
                 ORDER BY messages.timestamp DESC
                 LIMIT ?3 OFFSET ?4",
            )?;
            let msgs = stmt.query_map(rusqlite::params![query, ch_id, limit, offset], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    sender_peer_id: row.get(2)?,
                    sender_display_name: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                    edited_at: row.get(6)?,
                    deleted_at: row.get(7)?,
                    reply_to_id: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
            (total, msgs)
        } else {
            let total: i64 = conn.query_row(
                "SELECT COUNT(*)
                 FROM messages_fts
                 JOIN messages ON messages.rowid = messages_fts.rowid
                 WHERE messages_fts MATCH ?1 AND messages.deleted_at IS NULL",
                rusqlite::params![query],
                |row| row.get(0),
            )?;

            let mut stmt = conn.prepare(
                "SELECT messages.id, messages.channel_id, messages.sender_peer_id, messages.sender_display_name,
                        messages.content, messages.timestamp, messages.edited_at, messages.deleted_at, messages.reply_to_id
                 FROM messages_fts
                 JOIN messages ON messages.rowid = messages_fts.rowid
                 WHERE messages_fts MATCH ?1 AND messages.deleted_at IS NULL
                 ORDER BY messages.timestamp DESC
                 LIMIT ?2 OFFSET ?3",
            )?;
            let msgs = stmt.query_map(rusqlite::params![query, limit, offset], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    sender_peer_id: row.get(2)?,
                    sender_display_name: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                    edited_at: row.get(6)?,
                    deleted_at: row.get(7)?,
                    reply_to_id: row.get(8)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
            (total, msgs)
        };

        Ok(SearchResult { messages, total })
    }

    // ============================================================
    // Phase 2: Pinned Messages
    // ============================================================

    pub fn pin_message(&self, pin: &PinnedMessage) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO pinned_messages (id, channel_id, message_id, pinned_by, pinned_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                pin.id,
                pin.channel_id,
                pin.message_id,
                pin.pinned_by,
                pin.pinned_at,
            ],
        )?;
        Ok(())
    }

    pub fn unpin_message(&self, message_id: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "DELETE FROM pinned_messages WHERE message_id = ?1",
            rusqlite::params![message_id],
        )?;
        Ok(rows_affected > 0)
    }

    pub fn get_pinned_messages(&self, channel_id: &str) -> rusqlite::Result<Vec<PinnedMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, channel_id, message_id, pinned_by, pinned_at
             FROM pinned_messages
             WHERE channel_id = ?1
             ORDER BY pinned_at DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![channel_id], |row| {
            Ok(PinnedMessage {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                message_id: row.get(2)?,
                pinned_by: row.get(3)?,
                pinned_at: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    // ============================================================
    // Phase 2: DM Messages
    // ============================================================

    pub fn insert_dm_message(&self, id: &str, conversation_id: &str, sender_peer_id: &str, sender_display_name: &str, content: &str, timestamp: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO dm_messages (id, conversation_id, sender_peer_id, sender_display_name, content, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, conversation_id, sender_peer_id, sender_display_name, content, timestamp],
        )?;
        Ok(())
    }

    pub fn get_dm_messages(&self, conversation_id: &str, limit: i64, before: Option<&str>) -> rusqlite::Result<Vec<DmMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut messages = if let Some(before_ts) = before {
            let mut stmt = conn.prepare(
                "SELECT id, conversation_id, sender_peer_id, sender_display_name, content, timestamp
                 FROM dm_messages
                 WHERE conversation_id = ?1 AND timestamp < ?2
                 ORDER BY timestamp DESC LIMIT ?3",
            )?;
            let rows = stmt.query_map(rusqlite::params![conversation_id, before_ts, limit], |row| {
                Ok(DmMessage {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    sender_peer_id: row.get(2)?,
                    sender_display_name: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, conversation_id, sender_peer_id, sender_display_name, content, timestamp
                 FROM dm_messages
                 WHERE conversation_id = ?1
                 ORDER BY timestamp DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![conversation_id, limit], |row| {
                Ok(DmMessage {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    sender_peer_id: row.get(2)?,
                    sender_display_name: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        };
        messages.reverse();
        Ok(messages)
    }
}
