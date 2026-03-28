use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

pub const REACTIONS_MIGRATION: &str = r#"
    CREATE TABLE IF NOT EXISTS message_reactions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        message_id TEXT NOT NULL,
        sender_id TEXT NOT NULL,
        emoji TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(message_id, sender_id, emoji)
    );
    CREATE INDEX IF NOT EXISTS idx_reactions_message ON message_reactions(message_id);
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub message_id: String,
    pub sender_id: String,
    pub emoji: String,
}

pub fn add_reaction(conn: &Connection, reaction: &Reaction) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO message_reactions (message_id, sender_id, emoji) VALUES (?1, ?2, ?3)",
        params![reaction.message_id, reaction.sender_id, reaction.emoji],
    )?;
    Ok(())
}

pub fn remove_reaction(conn: &Connection, message_id: &str, sender_id: &str, emoji: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM message_reactions WHERE message_id = ?1 AND sender_id = ?2 AND emoji = ?3",
        params![message_id, sender_id, emoji],
    )?;
    Ok(())
}

pub fn get_reactions(conn: &Connection, message_id: &str) -> AppResult<Vec<Reaction>> {
    let mut stmt = conn.prepare("SELECT message_id, sender_id, emoji FROM message_reactions WHERE message_id = ?1")?;
    let rows = stmt.query_map(params![message_id], |row| {
        Ok(Reaction { message_id: row.get(0)?, sender_id: row.get(1)?, emoji: row.get(2)? })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_reactions_batch(
    conn: &Connection, message_ids: &[String],
) -> AppResult<std::collections::HashMap<String, Vec<Reaction>>> {
    if message_ids.is_empty() { return Ok(std::collections::HashMap::new()); }

    let placeholders: Vec<String> = (1..=message_ids.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT message_id, sender_id, emoji FROM message_reactions WHERE message_id IN ({})",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = message_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(Reaction { message_id: row.get(0)?, sender_id: row.get(1)?, emoji: row.get(2)? })
    })?;

    let mut result: std::collections::HashMap<String, Vec<Reaction>> = std::collections::HashMap::new();
    for row in rows.flatten() {
        result.entry(row.message_id.clone()).or_default().push(row);
    }
    Ok(result)
}
