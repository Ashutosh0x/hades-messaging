use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub message_id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub snippet: String,
    pub timestamp: String,
    pub rank: f64,
}

/// Index a message for full-text search (plaintext stored in FTS5, DB encrypted by SQLCipher)
pub fn index_message(conn: &Connection, message_id: &str, plaintext: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO messages_fts (message_id, search_tokens) VALUES (?1, ?2)",
        params![message_id, plaintext],
    )?;
    Ok(())
}

/// Search messages using FTS5 with BM25 ranking
pub fn search_messages(conn: &Connection, query: &str, limit: i64) -> AppResult<Vec<SearchResult>> {
    let safe_query = query.replace('"', "").replace('\'', "").replace(';', "");
    if safe_query.trim().is_empty() { return Ok(vec![]); }

    let sql = r#"
        SELECT f.message_id, m.conversation_id, m.sender_id,
               snippet(messages_fts, 1, '<b>', '</b>', '...', 32) as snippet,
               m.timestamp, rank
        FROM messages_fts f
        JOIN messages m ON m.id = f.message_id
        WHERE messages_fts MATCH ?1 AND m.is_deleted = 0
        ORDER BY rank LIMIT ?2
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![safe_query, limit], |row| {
        Ok(SearchResult {
            message_id: row.get(0)?, conversation_id: row.get(1)?,
            sender_id: row.get(2)?, snippet: row.get(3)?,
            timestamp: row.get(4)?, rank: row.get(5)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn remove_from_index(conn: &Connection, message_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM messages_fts WHERE message_id = ?1", params![message_id])?;
    Ok(())
}
