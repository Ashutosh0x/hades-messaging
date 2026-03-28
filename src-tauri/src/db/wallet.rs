use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Wallet migration SQL — added as v3
pub const WALLET_MIGRATION: &str = r#"
    CREATE TABLE IF NOT EXISTS wallet_accounts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        chain TEXT NOT NULL,
        address TEXT NOT NULL,
        derivation_path TEXT NOT NULL,
        public_key_hex TEXT NOT NULL,
        is_active INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(chain, derivation_path)
    );

    CREATE TABLE IF NOT EXISTS wallet_transactions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        tx_hash TEXT NOT NULL UNIQUE,
        chain TEXT NOT NULL,
        from_address TEXT NOT NULL,
        to_address TEXT NOT NULL,
        amount TEXT NOT NULL,
        symbol TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        explorer_url TEXT,
        message_id TEXT,
        conversation_id TEXT,
        timestamp INTEGER NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE INDEX IF NOT EXISTS idx_wallet_tx_chain
        ON wallet_transactions(chain, timestamp DESC);

    CREATE INDEX IF NOT EXISTS idx_wallet_tx_conversation
        ON wallet_transactions(conversation_id) WHERE conversation_id IS NOT NULL;

    CREATE TABLE IF NOT EXISTS wallet_tokens (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        chain TEXT NOT NULL,
        contract_address TEXT NOT NULL,
        symbol TEXT NOT NULL,
        name TEXT NOT NULL,
        decimals INTEGER NOT NULL,
        icon_url TEXT,
        is_visible INTEGER NOT NULL DEFAULT 1,
        UNIQUE(chain, contract_address)
    );

    CREATE TABLE IF NOT EXISTS wallet_custom_rpcs (
        chain TEXT PRIMARY KEY,
        rpc_url TEXT NOT NULL
    );
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccountRow {
    pub id: i64,
    pub chain: String,
    pub address: String,
    pub derivation_path: String,
    pub public_key_hex: String,
}

pub fn insert_account(conn: &Connection, acc: &WalletAccountRow) -> AppResult<()> {
    conn.execute(
        r#"INSERT OR IGNORE INTO wallet_accounts
           (chain, address, derivation_path, public_key_hex)
           VALUES (?1, ?2, ?3, ?4)"#,
        params![acc.chain, acc.address, acc.derivation_path, acc.public_key_hex],
    )?;
    Ok(())
}

pub fn get_all_accounts(conn: &Connection) -> AppResult<Vec<WalletAccountRow>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, chain, address, derivation_path, public_key_hex
           FROM wallet_accounts WHERE is_active = 1"#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(WalletAccountRow {
            id: row.get(0)?,
            chain: row.get(1)?,
            address: row.get(2)?,
            derivation_path: row.get(3)?,
            public_key_hex: row.get(4)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTxRow {
    pub tx_hash: String,
    pub chain: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub symbol: String,
    pub status: String,
    pub explorer_url: Option<String>,
    pub message_id: Option<String>,
    pub conversation_id: Option<String>,
    pub timestamp: i64,
}

pub fn insert_transaction(conn: &Connection, tx: &WalletTxRow) -> AppResult<()> {
    conn.execute(
        r#"INSERT OR REPLACE INTO wallet_transactions
           (tx_hash, chain, from_address, to_address, amount, symbol,
            status, explorer_url, message_id, conversation_id, timestamp)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
        params![
            tx.tx_hash,
            tx.chain,
            tx.from_address,
            tx.to_address,
            tx.amount,
            tx.symbol,
            tx.status,
            tx.explorer_url,
            tx.message_id,
            tx.conversation_id,
            tx.timestamp,
        ],
    )?;
    Ok(())
}

pub fn get_transactions(
    conn: &Connection,
    chain: Option<&str>,
    limit: i64,
) -> AppResult<Vec<WalletTxRow>> {
    let (sql, chain_param) = if let Some(chain) = chain {
        (
            r#"SELECT tx_hash, chain, from_address, to_address, amount, symbol,
                      status, explorer_url, message_id, conversation_id, timestamp
               FROM wallet_transactions WHERE chain = ?1
               ORDER BY timestamp DESC LIMIT ?2"#
                .to_string(),
            Some(chain.to_string()),
        )
    } else {
        (
            r#"SELECT tx_hash, chain, from_address, to_address, amount, symbol,
                      status, explorer_url, message_id, conversation_id, timestamp
               FROM wallet_transactions
               ORDER BY timestamp DESC LIMIT ?1"#
                .to_string(),
            None,
        )
    };

    let mut stmt = conn.prepare(&sql)?;

    let rows = if let Some(ref chain_val) = chain_param {
        stmt.query_map(params![chain_val, limit], |row| map_tx_row(row))?
    } else {
        stmt.query_map(params![limit], |row| map_tx_row(row))?
    };

    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn map_tx_row(row: &rusqlite::Row) -> rusqlite::Result<WalletTxRow> {
    Ok(WalletTxRow {
        tx_hash: row.get(0)?,
        chain: row.get(1)?,
        from_address: row.get(2)?,
        to_address: row.get(3)?,
        amount: row.get(4)?,
        symbol: row.get(5)?,
        status: row.get(6)?,
        explorer_url: row.get(7)?,
        message_id: row.get(8)?,
        conversation_id: row.get(9)?,
        timestamp: row.get(10)?,
    })
}

pub fn get_transactions_for_conversation(
    conn: &Connection,
    conversation_id: &str,
) -> AppResult<Vec<WalletTxRow>> {
    let mut stmt = conn.prepare(
        r#"SELECT tx_hash, chain, from_address, to_address, amount, symbol,
                  status, explorer_url, message_id, conversation_id, timestamp
           FROM wallet_transactions WHERE conversation_id = ?1
           ORDER BY timestamp DESC"#,
    )?;

    let rows = stmt.query_map(params![conversation_id], |row| map_tx_row(row))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn update_tx_status(conn: &Connection, tx_hash: &str, status: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE wallet_transactions SET status = ?1 WHERE tx_hash = ?2",
        params![status, tx_hash],
    )?;
    Ok(())
}
