pub mod attachments;
pub mod call;
pub mod contacts;
pub mod keys;
pub mod messages;
pub mod migrations;
pub mod pool;
pub mod reactions;
pub mod sessions;
pub mod wallet;

use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open(path: PathBuf, passphrase: &str) -> AppResult<Self> {
        let conn = Connection::open(&path)?;

        conn.pragma_update(None, "key", passphrase)?;
        conn.pragma_update(None, "cipher_page_size", "4096")?;
        conn.pragma_update(None, "kdf_iter", "256000")?;
        conn.pragma_update(None, "cipher_hmac_algorithm", "HMAC_SHA256")?;
        conn.pragma_update(None, "cipher_kdf_algorithm", "PBKDF2_HMAC_SHA256")?;

        // Performance pragmas
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", "-8000")?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;

        let mut stmt = conn.prepare("SELECT count(*) FROM sqlite_master")?;
        stmt.query_row([], |_| Ok(()))?;

        let mut db = Self { conn, path };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn rekey(&self, new_passphrase: &str) -> AppResult<()> {
        self.conn.pragma_update(None, "rekey", new_passphrase)?;
        Ok(())
    }

    pub fn destroy(self) -> AppResult<()> {
        let path = self.path.clone();
        drop(self.conn);
        if path.exists() {
            let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if len > 0 {
                let zeros = vec![0u8; len as usize];
                std::fs::write(&path, &zeros).ok();
            }
            std::fs::remove_file(&path).ok();
        }
        Ok(())
    }

    pub fn conn(&self) -> &Connection { &self.conn }

    fn run_migrations(&mut self) -> AppResult<()> { migrations::run_all(&self.conn) }
}
