use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

pub const ATTACHMENTS_MIGRATION: &str = r#"
    CREATE TABLE IF NOT EXISTS attachments (
        id TEXT PRIMARY KEY,
        message_id TEXT,
        file_name TEXT NOT NULL,
        file_size INTEGER NOT NULL,
        mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
        media_type INTEGER NOT NULL DEFAULT 0,
        digest TEXT NOT NULL,
        width INTEGER,
        height INTEGER,
        duration_secs REAL,
        thumbnail BLOB,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        last_access TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_attachments_message
        ON attachments(message_id);

    CREATE TABLE IF NOT EXISTS attachment_parts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        attachment_id TEXT NOT NULL,
        part_index INTEGER NOT NULL,
        chunk_data BLOB NOT NULL,
        iv BLOB NOT NULL,
        mac BLOB NOT NULL,
        UNIQUE(attachment_id, part_index),
        FOREIGN KEY (attachment_id) REFERENCES attachments(id) ON DELETE CASCADE
    );
"#;

/// Media type enum matching WhatsApp conventions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i32)]
pub enum MediaType {
    Photo = 0,
    Video = 1,
    Audio = 2,
    VoiceNote = 3,
    Document = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentRow {
    pub id: String,
    pub message_id: Option<String>,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub media_type: i32,
    pub digest: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_secs: Option<f64>,
    pub created_at: String,
}

pub fn insert_attachment(conn: &Connection, att: &AttachmentRow) -> AppResult<()> {
    conn.execute(
        r#"INSERT INTO attachments
            (id, message_id, file_name, file_size, mime_type, media_type, digest, width, height, duration_secs)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        params![
            att.id,
            att.message_id,
            att.file_name,
            att.file_size,
            att.mime_type,
            att.media_type,
            att.digest,
            att.width,
            att.height,
            att.duration_secs,
        ],
    )?;
    Ok(())
}

pub fn insert_chunk(
    conn: &Connection,
    attachment_id: &str,
    part_index: i32,
    chunk_data: &[u8],
    iv: &[u8],
    mac: &[u8],
) -> AppResult<()> {
    conn.execute(
        r#"INSERT INTO attachment_parts (attachment_id, part_index, chunk_data, iv, mac)
           VALUES (?1, ?2, ?3, ?4, ?5)"#,
        params![attachment_id, part_index, chunk_data, iv, mac],
    )?;
    Ok(())
}

pub fn get_attachment(conn: &Connection, id: &str) -> AppResult<Option<AttachmentRow>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, message_id, file_name, file_size, mime_type, media_type, digest,
                  width, height, duration_secs, created_at
           FROM attachments WHERE id = ?1"#,
    )?;

    let result = stmt
        .query_row(params![id], |row| {
            Ok(AttachmentRow {
                id: row.get(0)?,
                message_id: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                mime_type: row.get(4)?,
                media_type: row.get(5)?,
                digest: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                duration_secs: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .ok();

    Ok(result)
}

pub fn get_chunks(conn: &Connection, attachment_id: &str) -> AppResult<Vec<Vec<u8>>> {
    let mut stmt = conn.prepare(
        "SELECT chunk_data FROM attachment_parts WHERE attachment_id = ?1 ORDER BY part_index ASC",
    )?;

    let rows = stmt.query_map(params![attachment_id], |row| {
        let data: Vec<u8> = row.get(0)?;
        Ok(data)
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_attachments_for_conversation(
    conn: &Connection,
    conversation_id: &str,
    media_type: Option<i32>,
) -> AppResult<Vec<AttachmentRow>> {
    let (sql, use_type) = match media_type {
        Some(mt) => (
            r#"SELECT a.id, a.message_id, a.file_name, a.file_size, a.mime_type,
                      a.media_type, a.digest, a.width, a.height, a.duration_secs, a.created_at
               FROM attachments a
               JOIN messages m ON a.message_id = m.id
               WHERE m.conversation_id = ?1 AND a.media_type = ?2
               ORDER BY a.created_at DESC"#,
            Some(mt),
        ),
        None => (
            r#"SELECT a.id, a.message_id, a.file_name, a.file_size, a.mime_type,
                      a.media_type, a.digest, a.width, a.height, a.duration_secs, a.created_at
               FROM attachments a
               JOIN messages m ON a.message_id = m.id
               WHERE m.conversation_id = ?1
               ORDER BY a.created_at DESC"#,
            None,
        ),
    };

    let mut stmt = conn.prepare(sql)?;

    let rows = if let Some(mt) = use_type {
        stmt.query_map(params![conversation_id, mt], |row| {
            Ok(AttachmentRow {
                id: row.get(0)?,
                message_id: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                mime_type: row.get(4)?,
                media_type: row.get(5)?,
                digest: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                duration_secs: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?
    } else {
        stmt.query_map(params![conversation_id], |row| {
            Ok(AttachmentRow {
                id: row.get(0)?,
                message_id: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                mime_type: row.get(4)?,
                media_type: row.get(5)?,
                digest: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                duration_secs: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?
    };

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn delete_attachment(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM attachment_parts WHERE attachment_id = ?1", params![id])?;
    conn.execute("DELETE FROM attachments WHERE id = ?1", params![id])?;
    Ok(())
}
