use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Semaphore, Mutex};

const POOL_SIZE: usize = 4;

/// Thread-safe connection pool for SQLCipher
pub struct ConnectionPool {
    connections: Vec<Arc<Mutex<Connection>>>,
    semaphore: Arc<Semaphore>,
    #[allow(dead_code)]
    path: PathBuf,
}

impl ConnectionPool {
    pub fn open(path: PathBuf, passphrase: &str) -> AppResult<Self> {
        let mut connections = Vec::with_capacity(POOL_SIZE);

        for i in 0..POOL_SIZE {
            let conn = Connection::open(&path)?;

            // SQLCipher config
            conn.pragma_update(None, "key", passphrase)?;
            conn.pragma_update(None, "cipher_page_size", "4096")?;
            conn.pragma_update(None, "kdf_iter", "256000")?;

            // Performance pragmas
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "cache_size", "-8000")?; // 8MB
            conn.pragma_update(None, "temp_store", "MEMORY")?;
            conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

            // Verify key on first connection
            if i == 0 {
                conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;
            }

            connections.push(Arc::new(Mutex::new(conn)));
        }

        // Run migrations on first connection
        {
            let first = connections[0].blocking_lock();
            crate::db::migrations::run_all(&first)?;
        }

        Ok(Self {
            connections,
            semaphore: Arc::new(Semaphore::new(POOL_SIZE)),
            path,
        })
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> AppResult<PooledConnection<'_>> {
        let permit = self.semaphore.acquire().await
            .map_err(|_| AppError::Internal("Pool exhausted".into()))?;

        for conn in &self.connections {
            if let Ok(guard) = conn.try_lock() {
                return Ok(PooledConnection { conn: guard, _permit: permit });
            }
        }

        let guard = self.connections[0].lock().await;
        Ok(PooledConnection { conn: guard, _permit: permit })
    }

    /// Execute a read query on any available connection
    pub async fn read<F, T>(&self, f: F) -> AppResult<T>
    where F: FnOnce(&Connection) -> AppResult<T> {
        let conn = self.acquire().await?;
        f(&conn.conn)
    }

    /// Execute a write query (serialized through first connection)
    pub async fn write<F, T>(&self, f: F) -> AppResult<T>
    where F: FnOnce(&Connection) -> AppResult<T> {
        let guard = self.connections[0].lock().await;
        f(&guard)
    }
}

pub struct PooledConnection<'a> {
    conn: tokio::sync::MutexGuard<'a, Connection>,
    _permit: tokio::sync::SemaphorePermit<'a>,
}

impl<'a> std::ops::Deref for PooledConnection<'a> {
    type Target = Connection;
    fn deref(&self) -> &Connection { &self.conn }
}
