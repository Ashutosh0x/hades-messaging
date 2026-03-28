use crate::error::AppResult;
use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[
    // v1: Core tables
    r#"
    CREATE TABLE IF NOT EXISTS identity (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        public_key BLOB NOT NULL,
        secret_key_encrypted BLOB NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS contacts (
        id TEXT PRIMARY KEY,
        display_name TEXT NOT NULL,
        identity_key BLOB NOT NULL,
        safety_number TEXT,
        verified INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        conversation_id TEXT NOT NULL,
        sender_id TEXT NOT NULL,
        content_encrypted BLOB NOT NULL,
        content_nonce BLOB NOT NULL,
        timestamp TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'sending',
        burn_after INTEGER,
        reply_to TEXT,
        is_deleted INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE INDEX IF NOT EXISTS idx_messages_conversation
        ON messages(conversation_id, timestamp DESC);

    CREATE INDEX IF NOT EXISTS idx_messages_status
        ON messages(status) WHERE status != 'read';

    CREATE TABLE IF NOT EXISTS ratchet_sessions (
        contact_id TEXT PRIMARY KEY,
        session_data BLOB NOT NULL,
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS prekeys (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        public_key BLOB NOT NULL,
        secret_key_encrypted BLOB NOT NULL,
        is_consumed INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS signed_prekeys (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        public_key BLOB NOT NULL,
        secret_key_encrypted BLOB NOT NULL,
        signature BLOB NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS devices (
        device_id TEXT PRIMARY KEY,
        device_name TEXT NOT NULL,
        identity_key BLOB NOT NULL,
        last_seen TEXT,
        is_revoked INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS kv_store (
        key TEXT PRIMARY KEY,
        value BLOB NOT NULL
    );
    "#,
    // v2: FTS5 for encrypted search index
    r#"
    CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
        message_id,
        search_tokens,
        content='',
        tokenize='unicode61'
    );
    "#,
    // v3: Wallet tables
    super::wallet::WALLET_MIGRATION,
    // v4: Message reactions
    super::reactions::REACTIONS_MIGRATION,
    // v5: Group messaging tables
    r#"
    CREATE TABLE IF NOT EXISTS groups (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        avatar BLOB,
        created_by TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS group_members (
        group_id TEXT NOT NULL,
        member_id TEXT NOT NULL,
        role TEXT NOT NULL DEFAULT 'member',
        joined_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (group_id, member_id),
        FOREIGN KEY (group_id) REFERENCES groups(id)
    );

    CREATE TABLE IF NOT EXISTS group_sender_keys (
        group_id TEXT NOT NULL,
        member_id TEXT NOT NULL,
        sender_key_data BLOB NOT NULL,
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (group_id, member_id)
    );
    "#,
];

pub fn run_all(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY);",
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i64;
        if version > current_version {
            log::info!("Running migration v{}", version);
            conn.execute_batch(migration)?;
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [version],
            )?;
        }
    }

    Ok(())
}
