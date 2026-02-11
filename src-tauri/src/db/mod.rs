pub mod messages;
pub mod rooms;

use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir).ok();
        let db_path = data_dir.join("chatr.db");
        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        db.run_migrations()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            );

            CREATE TABLE IF NOT EXISTS identity (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                keypair_bytes BLOB NOT NULL,
                display_name TEXT NOT NULL DEFAULT 'Anonymous',
                avatar_hash TEXT,
                status_message TEXT,
                status_type TEXT DEFAULT 'online'
            );

            CREATE TABLE IF NOT EXISTS rooms (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                invite_code TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                owner_peer_id TEXT
            );

            CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL REFERENCES rooms(id),
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                channel_type TEXT NOT NULL DEFAULT 'text',
                topic TEXT,
                position INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL REFERENCES channels(id),
                sender_peer_id TEXT NOT NULL,
                sender_display_name TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                edited_at TEXT,
                deleted_at TEXT,
                reply_to_id TEXT
            );

            CREATE TABLE IF NOT EXISTS reactions (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL REFERENCES messages(id),
                peer_id TEXT NOT NULL,
                emoji TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(message_id, peer_id, emoji)
            );

            CREATE TABLE IF NOT EXISTS read_receipts (
                channel_id TEXT NOT NULL,
                peer_id TEXT NOT NULL,
                last_read_message_id TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (channel_id, peer_id)
            );

            CREATE TABLE IF NOT EXISTS dm_conversations (
                id TEXT PRIMARY KEY,
                is_group INTEGER NOT NULL DEFAULT 0,
                name TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS dm_participants (
                conversation_id TEXT NOT NULL REFERENCES dm_conversations(id),
                peer_id TEXT NOT NULL,
                joined_at TEXT NOT NULL,
                PRIMARY KEY (conversation_id, peer_id)
            );

            CREATE TABLE IF NOT EXISTS dm_messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL REFERENCES dm_conversations(id),
                sender_peer_id TEXT NOT NULL,
                sender_display_name TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pinned_messages (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                message_id TEXT NOT NULL UNIQUE,
                pinned_by TEXT NOT NULL,
                pinned_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS room_roles (
                id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL REFERENCES rooms(id),
                peer_id TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'member',
                assigned_by TEXT NOT NULL,
                assigned_at TEXT NOT NULL,
                UNIQUE(room_id, peer_id)
            );

            CREATE TABLE IF NOT EXISTS moderation_actions (
                id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL REFERENCES rooms(id),
                action_type TEXT NOT NULL,
                target_peer_id TEXT NOT NULL,
                moderator_peer_id TEXT NOT NULL,
                reason TEXT,
                created_at TEXT NOT NULL,
                expires_at TEXT
            );

            CREATE TABLE IF NOT EXISTS blocked_peers (
                peer_id TEXT PRIMARY KEY,
                blocked_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                size INTEGER NOT NULL,
                mime_type TEXT NOT NULL,
                sha256_hash TEXT NOT NULL,
                chunk_count INTEGER NOT NULL,
                uploader_peer_id TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS message_attachments (
                message_id TEXT NOT NULL,
                file_id TEXT NOT NULL REFERENCES files(id),
                PRIMARY KEY (message_id, file_id)
            );

            CREATE TABLE IF NOT EXISTS friends (
                peer_id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending_outgoing',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS custom_emoji (
                id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL REFERENCES rooms(id),
                name TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                uploaded_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(room_id, name)
            );

            CREATE TABLE IF NOT EXISTS notification_settings (
                target_id TEXT NOT NULL,
                target_type TEXT NOT NULL,
                level TEXT NOT NULL DEFAULT 'all',
                PRIMARY KEY (target_id, target_type)
            );

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_channels_room ON channels(room_id);
            CREATE INDEX IF NOT EXISTS idx_reactions_message ON reactions(message_id);
            CREATE INDEX IF NOT EXISTS idx_dm_messages_conv ON dm_messages(conversation_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_moderation_room ON moderation_actions(room_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_pinned_channel ON pinned_messages(channel_id);
            CREATE INDEX IF NOT EXISTS idx_files_hash ON files(sha256_hash);

            -- FTS5 for full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                id UNINDEXED,
                channel_id UNINDEXED,
                sender_display_name,
                content,
                content=messages,
                content_rowid=rowid
            );

            -- Triggers to keep FTS in sync
            CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(rowid, id, channel_id, sender_display_name, content)
                VALUES (new.rowid, new.id, new.channel_id, new.sender_display_name, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, id, channel_id, sender_display_name, content)
                VALUES ('delete', old.rowid, old.id, old.channel_id, old.sender_display_name, old.content);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, id, channel_id, sender_display_name, content)
                VALUES ('delete', old.rowid, old.id, old.channel_id, old.sender_display_name, old.content);
                INSERT INTO messages_fts(rowid, id, channel_id, sender_display_name, content)
                VALUES (new.rowid, new.id, new.channel_id, new.sender_display_name, new.content);
            END;
            ",
        )?;
        Ok(())
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if version < 1 {
            conn.execute("INSERT OR REPLACE INTO schema_version (version) VALUES (1)", [])?;
        }

        Ok(())
    }

    pub fn save_keypair(&self, keypair_bytes: &[u8]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO identity (id, keypair_bytes, display_name)
             VALUES (1, ?1, COALESCE((SELECT display_name FROM identity WHERE id = 1), 'Anonymous'))",
            [keypair_bytes],
        )?;
        Ok(())
    }

    pub fn load_keypair(&self) -> Result<Option<Vec<u8>>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT keypair_bytes FROM identity WHERE id = 1")?;
        let result = stmt.query_row([], |row| row.get(0));
        match result {
            Ok(bytes) => Ok(Some(bytes)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_display_name(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT display_name FROM identity WHERE id = 1")?;
        let result = stmt.query_row([], |row| row.get::<_, String>(0));
        match result {
            Ok(name) => Ok(name),
            Err(_) => Ok("Anonymous".to_string()),
        }
    }

    pub fn set_display_name(&self, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE identity SET display_name = ?1 WHERE id = 1",
            [name],
        )?;
        Ok(())
    }

    pub fn get_identity_profile(&self) -> Result<(String, Option<String>, Option<String>, Option<String>)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT display_name, avatar_hash, status_message, status_type FROM identity WHERE id = 1"
        )?;
        let result = stmt.query_row([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        });
        match result {
            Ok(v) => Ok(v),
            Err(_) => Ok(("Anonymous".to_string(), None, None, None)),
        }
    }

    pub fn set_status(&self, status_message: Option<&str>, status_type: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE identity SET status_message = ?1, status_type = ?2 WHERE id = 1",
            rusqlite::params![status_message, status_type],
        )?;
        Ok(())
    }

    pub fn set_avatar_hash(&self, hash: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE identity SET avatar_hash = ?1 WHERE id = 1",
            [hash],
        )?;
        Ok(())
    }

    // ============================================================
    // Settings (Phase 6)
    // ============================================================

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        match stmt.query_row([key], |row| row.get::<_, String>(0)) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn delete_setting(&self, key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
        Ok(())
    }

    // ============================================================
    // Notification Settings (Phase 7)
    // ============================================================

    pub fn get_notification_setting(&self, target_id: &str, target_type: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT level FROM notification_settings WHERE target_id = ?1 AND target_type = ?2"
        )?;
        match stmt.query_row(rusqlite::params![target_id, target_type], |row| row.get::<_, String>(0)) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_notification_setting(&self, target_id: &str, target_type: &str, level: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO notification_settings (target_id, target_type, level) VALUES (?1, ?2, ?3)",
            rusqlite::params![target_id, target_type, level],
        )?;
        Ok(())
    }

    pub fn get_all_notification_settings(&self) -> Result<Vec<crate::models::NotificationSetting>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT target_id, target_type, level FROM notification_settings")?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::models::NotificationSetting {
                target_id: row.get(0)?,
                target_type: row.get(1)?,
                level: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }
}
