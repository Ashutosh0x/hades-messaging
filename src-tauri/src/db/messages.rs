use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content_encrypted: Vec<u8>,
    pub content_nonce: Vec<u8>,
    pub timestamp: String,
    pub status: String,
    pub burn_after: Option<i64>,
    pub reply_to: Option<String>,
}

pub fn insert_message(conn: &Connection, msg: &StoredMessage) -> AppResult<()> {
    conn.execute(
        r#"INSERT INTO messages
            (id, conversation_id, sender_id, content_encrypted,
             content_nonce, timestamp, status, burn_after, reply_to)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
        params![
            msg.id,
            msg.conversation_id,
            msg.sender_id,
            msg.content_encrypted,
            msg.content_nonce,
            msg.timestamp,
            msg.status,
            msg.burn_after,
            msg.reply_to,
        ],
    )?;
    Ok(())
}

pub fn get_messages_for_conversation(
    conn: &Connection,
    conversation_id: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<StoredMessage>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, conversation_id, sender_id, content_encrypted,
                  content_nonce, timestamp, status, burn_after, reply_to
           FROM messages
           WHERE conversation_id = ?1 AND is_deleted = 0
           ORDER BY timestamp DESC
           LIMIT ?2 OFFSET ?3"#,
    )?;

    let rows = stmt.query_map(params![conversation_id, limit, offset], |row| {
        Ok(StoredMessage {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            sender_id: row.get(2)?,
            content_encrypted: row.get(3)?,
            content_nonce: row.get(4)?,
            timestamp: row.get(5)?,
            status: row.get(6)?,
            burn_after: row.get(7)?,
            reply_to: row.get(8)?,
        })
    })?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    Ok(messages)
}

pub fn update_message_status(
    conn: &Connection,
    message_id: &str,
    status: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE messages SET status = ?1 WHERE id = ?2",
        params![status, message_id],
    )?;
    Ok(())
}

pub fn delete_message(conn: &Connection, message_id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE messages SET is_deleted = 1, content_encrypted = zeroblob(0), content_nonce = zeroblob(0) WHERE id = ?1",
        params![message_id],
    )?;
    Ok(())
}

pub fn delete_expired_burn_messages(conn: &Connection) -> AppResult<u64> {
    let count = conn.execute(
        r#"UPDATE messages
           SET is_deleted = 1,
               content_encrypted = zeroblob(0),
               content_nonce = zeroblob(0)
           WHERE burn_after IS NOT NULL
             AND is_deleted = 0
             AND datetime(timestamp, '+' || burn_after || ' seconds') < datetime('now')"#,
        [],
    )?;
    Ok(count as u64)
}

// ─── Cursor-Based Pagination ──────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MessagePage {
    pub messages: Vec<StoredMessage>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub total_count: i64,
}

pub fn get_messages_paginated(
    conn: &Connection,
    conversation_id: &str,
    cursor: Option<&str>,
    limit: i64,
    direction: &str,
) -> AppResult<MessagePage> {
    let (where_clause, order) = match (cursor, direction) {
        (Some(ts), "older") => (format!("AND timestamp < '{}'", ts), "DESC"),
        (Some(ts), "newer") => (format!("AND timestamp > '{}'", ts), "ASC"),
        _ => (String::new(), "DESC"),
    };

    let sql = format!(
        r#"SELECT id, conversation_id, sender_id, content_encrypted,
                  content_nonce, timestamp, status, burn_after, reply_to
           FROM messages
           WHERE conversation_id = ?1 AND is_deleted = 0 {}
           ORDER BY timestamp {}
           LIMIT ?2"#,
        where_clause, order
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![conversation_id, limit + 1], |row| {
        Ok(StoredMessage {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            sender_id: row.get(2)?,
            content_encrypted: row.get(3)?,
            content_nonce: row.get(4)?,
            timestamp: row.get(5)?,
            status: row.get(6)?,
            burn_after: row.get(7)?,
            reply_to: row.get(8)?,
        })
    })?;

    let mut messages: Vec<StoredMessage> = rows.filter_map(|r| r.ok()).collect();
    let has_more = messages.len() > limit as usize;
    if has_more { messages.truncate(limit as usize); }

    let next_cursor = messages.last().map(|m| m.timestamp.clone());

    let total_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE conversation_id = ?1 AND is_deleted = 0",
        params![conversation_id],
        |row| row.get(0),
    ).unwrap_or(0);

    Ok(MessagePage { messages, has_more, next_cursor, total_count })
}
