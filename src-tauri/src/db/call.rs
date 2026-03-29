use crate::error::AppResult;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

pub const CALLS_MIGRATION: &str = r#"
    CREATE TABLE IF NOT EXISTS call_history (
        id TEXT NOT NULL PRIMARY KEY,
        conversation_id TEXT NOT NULL,
        timestamp INTEGER NOT NULL,
        call_type INTEGER NOT NULL,     -- 0: voice, 1: video
        direction INTEGER NOT NULL,     -- 0: incoming, 1: outgoing, 2: missed
        status INTEGER NOT NULL,        -- 0: initiated, 1: accepted, 2: rejected, 3: connected, 4: ended
        duration INTEGER,
        extra_data BLOB,                -- Signal Client internal state (protobufs)
        signal_call_id TEXT,            -- Unique session identifier
        network_type TEXT,              -- "wifi", "p2p", "websocket"
        ice_candidates BLOB,            -- Signal Client ICE candidates (JSON array)
        is_silence INTEGER DEFAULT 0,   -- Screen share / Mute status
        is_video_call INTEGER DEFAULT 1,
        video_codec TEXT DEFAULT "",    -- "VP8", "H264"
        audio_codec TEXT DEFAULT "opus",-- "OPUS"
        created_at INTEGER DEFAULT (cast(strftime('%s', 'now') as integer)),
        updated_at INTEGER DEFAULT (cast(strftime('%s', 'now') as integer))
    );

    CREATE TABLE IF NOT EXISTS call_ice_candidates (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        call_id TEXT NOT NULL,
        connection_id TEXT NOT NULL,
        type INTEGER,                   -- 0: host, 1: srflx, 2: turn
        host_address TEXT,
        stun_address TEXT,
        turn_address TEXT,
        username TEXT,
        credential_bytes BLOB,
        priority INTEGER DEFAULT 0,
        FOREIGN KEY (call_id) REFERENCES call_history(id) ON DELETE CASCADE
    );
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallHistoryRow {
    pub id: String,
    pub conversation_id: String,
    pub timestamp: i64,
    pub call_type: i32,
    pub direction: i32,
    pub status: i32,
    pub duration: i64,
    pub extra_data: Vec<u8>,
    pub network_type: String,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallIceCandidate {
    pub id: i64,
    pub call_id: String,
    pub connection_id: String,
    pub r#type: i32,
    pub host_address: String,
    pub stun_address: Option<String>,
    pub turn_address: Option<String>,
    pub username: Option<String>,
    pub hades_id: String, 
    pub credential_bytes: Vec<u8>,
    pub priority: i32,
}

impl CallHistoryRow {
    pub fn new(id: String, conversation_id: String, call_type: i32, direction: i32, timestamp: i64) -> Self {
        Self {
            id,
            conversation_id,
            timestamp,
            call_type,
            direction,
            status: 0, 
            duration: 0,
            extra_data: Vec::new(),
            network_type: "websocket".to_string(),
            updated_at: Some(unix_timestamp()),
        }
    }

    pub fn update_status(&mut self, new_status: i32, duration: i64) {
        self.status = new_status;
        self.duration = duration;
        self.updated_at = Some(unix_timestamp());
    }

    pub fn set_signal_call_data(&mut self, data: Vec<u8>) {
        self.extra_data = data;
    }
}

pub fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn call_insert(conn: &Connection, call: &CallHistoryRow) -> rusqlite::Result<()> {
    conn.execute(
        r#"
        INSERT INTO call_history (
            id, conversation_id, timestamp, call_type, direction, status, duration, extra_data, network_type
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            call.id,
            call.conversation_id,
            call.timestamp,
            call.call_type,
            call.direction,
            call.status,
            call.duration,
            call.extra_data,
            call.network_type,
        ],
    )?;
    Ok(())
}

pub fn call_update(conn: &Connection, id: &str, status: i32, duration: i64) -> rusqlite::Result<()> {
    conn.execute(
        r#"
        UPDATE call_history SET status = ?1, duration = ?2, updated_at = ?3 WHERE id = ?4
        "#,
        params![
            status,
            duration,
            unix_timestamp(),
            id
        ],
    )?;
    Ok(())
}

pub fn get_calls(conn: &Connection) -> rusqlite::Result<Vec<CallHistoryRow>> {
    let mut stmt = conn.prepare("SELECT id, conversation_id, timestamp, call_type, direction, status, duration, extra_data, network_type FROM call_history ORDER BY timestamp DESC")?;
    let call_iter = stmt.query_map([], |row| {
        Ok(CallHistoryRow {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            timestamp: row.get(2)?,
            call_type: row.get(3)?,
            direction: row.get(4)?,
            status: row.get(5)?,
            duration: row.get(6)?,
            extra_data: row.get(7)?,
            network_type: row.get(8).unwrap_or_else(|_| "websocket".to_string()),
            updated_at: None,
        })
    })?;

    let mut calls = Vec::new();
    for call in call_iter {
        calls.push(call?);
    }
    Ok(calls)
}
