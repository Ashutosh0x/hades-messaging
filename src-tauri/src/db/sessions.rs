use crate::error::AppResult;
use rusqlite::{params, Connection};

/// Serialize and store a ratchet session
pub fn save_session(
    conn: &Connection,
    contact_id: &str,
    session_data: &[u8],
) -> AppResult<()> {
    conn.execute(
        r#"INSERT INTO ratchet_sessions (contact_id, session_data, updated_at)
           VALUES (?1, ?2, datetime('now'))
           ON CONFLICT(contact_id) DO UPDATE SET
               session_data = excluded.session_data,
               updated_at = excluded.updated_at"#,
        params![contact_id, session_data],
    )?;
    Ok(())
}

/// Load a ratchet session
pub fn load_session(
    conn: &Connection,
    contact_id: &str,
) -> AppResult<Option<Vec<u8>>> {
    let mut stmt = conn.prepare(
        "SELECT session_data FROM ratchet_sessions WHERE contact_id = ?1",
    )?;

    let result = stmt
        .query_row(params![contact_id], |row| row.get::<_, Vec<u8>>(0))
        .ok();

    Ok(result)
}

pub fn delete_session(conn: &Connection, contact_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM ratchet_sessions WHERE contact_id = ?1",
        params![contact_id],
    )?;
    Ok(())
}

pub fn has_session(conn: &Connection, contact_id: &str) -> AppResult<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ratchet_sessions WHERE contact_id = ?1",
        params![contact_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}
