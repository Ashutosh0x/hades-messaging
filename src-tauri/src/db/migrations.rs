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
    // v6: Call history (Q1 fix) -> delegated to call.rs
    super::call::CALLS_MIGRATION,
    // v7: Attachment chunking (WhatsApp-style media parts)
    super::attachments::ATTACHMENTS_MIGRATION,
    // v8: WhatsApp-level schema completeness + performance indexes
    r#"
    -- FTS5 auto-update triggers (auto-index on message insert/delete/update)
    CREATE TRIGGER IF NOT EXISTS fts_insert AFTER INSERT ON messages BEGIN
        INSERT INTO messages_fts(message_id, search_tokens)
        VALUES (new.id, '');
    END;

    CREATE TRIGGER IF NOT EXISTS fts_delete AFTER DELETE ON messages BEGIN
        INSERT INTO messages_fts(messages_fts, message_id, search_tokens)
        VALUES ('delete', old.id, '');
    END;

    -- Conversation statistics cache (O(1) badge counts + preview)
    CREATE TABLE IF NOT EXISTS conversation_stats (
        conversation_id TEXT PRIMARY KEY,
        message_count INTEGER NOT NULL DEFAULT 0,
        media_count INTEGER NOT NULL DEFAULT 0,
        unread_count INTEGER NOT NULL DEFAULT 0,
        last_message_timestamp TEXT,
        first_message_timestamp TEXT,
        total_bytes INTEGER NOT NULL DEFAULT 0,
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    -- Message edits (WhatsApp edit feature)
    CREATE TABLE IF NOT EXISTS message_edits (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        message_id TEXT NOT NULL,
        edit_version INTEGER NOT NULL,
        previous_content BLOB,
        edited_at TEXT NOT NULL DEFAULT (datetime('now')),
        FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
    );

    -- Dedicated message receipts (delivery + read)
    CREATE TABLE IF NOT EXISTS message_receipts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        message_id TEXT NOT NULL,
        recipient_id TEXT NOT NULL,
        receipt_type TEXT NOT NULL,
        timestamp TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(message_id, recipient_id, receipt_type),
        FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
    );

    -- Draft messages (unsent draft per conversation)
    CREATE TABLE IF NOT EXISTS message_drafts (
        conversation_id TEXT PRIMARY KEY,
        draft_text TEXT,
        draft_attachments TEXT,
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    -- Starred messages
    CREATE TABLE IF NOT EXISTS starred_messages (
        message_id TEXT PRIMARY KEY,
        starred_at TEXT NOT NULL DEFAULT (datetime('now')),
        FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
    );

    -- Pinned conversations
    CREATE TABLE IF NOT EXISTS pinned_conversations (
        conversation_id TEXT PRIMARY KEY,
        pin_order INTEGER NOT NULL DEFAULT 0,
        pinned_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    -- Muted conversations
    CREATE TABLE IF NOT EXISTS muted_conversations (
        conversation_id TEXT PRIMARY KEY,
        muted_until INTEGER NOT NULL,
        muted_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    -- Archived conversations
    CREATE TABLE IF NOT EXISTS archived_conversations (
        conversation_id TEXT PRIMARY KEY,
        archived_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    -- Performance indexes
    CREATE INDEX IF NOT EXISTS idx_messages_unread
        ON messages(conversation_id, status)
        WHERE status != 'read' AND sender_id != 'self';

    CREATE INDEX IF NOT EXISTS idx_messages_burn
        ON messages(burn_after)
        WHERE burn_after IS NOT NULL AND is_deleted = 0;

    CREATE INDEX IF NOT EXISTS idx_call_history_contact
        ON call_history(contact_id, timestamp DESC);

    CREATE INDEX IF NOT EXISTS idx_receipts_message
        ON message_receipts(message_id);

    -- SQLite performance tuning
    PRAGMA auto_vacuum = INCREMENTAL;
    PRAGMA mmap_size = 268435456;
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
