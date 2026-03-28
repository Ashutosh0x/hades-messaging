use crate::error::AppResult;
use rusqlite::{params, Connection};

/// Store a batch of one-time prekeys
pub fn insert_prekeys(
    conn: &Connection,
    prekeys: &[(Vec<u8>, Vec<u8>)], // (public, secret_encrypted)
) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    for (pub_key, sec_key) in prekeys {
        tx.execute(
            "INSERT INTO prekeys (public_key, secret_key_encrypted) VALUES (?1, ?2)",
            params![pub_key, sec_key],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Consume one prekey (mark used, return secret key)
pub fn consume_prekey(
    conn: &Connection,
    public_key: &[u8],
) -> AppResult<Option<Vec<u8>>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, secret_key_encrypted FROM prekeys
           WHERE public_key = ?1 AND is_consumed = 0
           LIMIT 1"#,
    )?;

    let result = stmt
        .query_row(params![public_key], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .ok();

    if let Some((id, secret)) = result {
        conn.execute(
            "UPDATE prekeys SET is_consumed = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(Some(secret))
    } else {
        Ok(None)
    }
}

/// Get count of available (unconsumed) prekeys
pub fn available_prekey_count(conn: &Connection) -> AppResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM prekeys WHERE is_consumed = 0",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Store the signed prekey
pub fn upsert_signed_prekey(
    conn: &Connection,
    public_key: &[u8],
    secret_key_encrypted: &[u8],
    signature: &[u8],
) -> AppResult<()> {
    conn.execute(
        r#"INSERT INTO signed_prekeys (public_key, secret_key_encrypted, signature)
           VALUES (?1, ?2, ?3)"#,
        params![public_key, secret_key_encrypted, signature],
    )?;
    Ok(())
}

/// Store the identity in the single-row identity table
pub fn store_identity(
    conn: &Connection,
    public_key: &[u8],
    secret_key_encrypted: &[u8],
) -> AppResult<()> {
    conn.execute(
        r#"INSERT INTO identity (id, public_key, secret_key_encrypted)
           VALUES (1, ?1, ?2)
           ON CONFLICT(id) DO UPDATE SET
               public_key = excluded.public_key,
               secret_key_encrypted = excluded.secret_key_encrypted"#,
        params![public_key, secret_key_encrypted],
    )?;
    Ok(())
}

pub fn load_identity(
    conn: &Connection,
) -> AppResult<Option<(Vec<u8>, Vec<u8>)>> {
    let mut stmt = conn.prepare(
        "SELECT public_key, secret_key_encrypted FROM identity WHERE id = 1",
    )?;

    let result = stmt
        .query_row([], |row| {
            Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .ok();

    Ok(result)
}
