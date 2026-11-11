//! User accounts and search history in a small SQLite file (`users.db`).
//! The table layout is exactly the one the old FastAPI backend created, so existing
//! accounts, bcrypt password hashes and history keep working.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;

pub const ROLES: [&str; 2] = ["admin", "user"];
const BCRYPT_COST: u32 = 12;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub active: bool,
    #[serde(skip)]
    pub token_version: i64,
    pub created_at: String,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

#[derive(Debug, Serialize)]
pub struct HistoryItem {
    pub query: String,
    pub mode: String,
    pub results: i64,
    pub created_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("{0}")]
    Invalid(String),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("password hashing failed: {0}")]
    Hash(#[from] bcrypt::BcryptError),
}

/// bcrypt only uses the first 72 bytes (the old Python backend cut it the same way).
fn pw72(password: &str) -> &[u8] {
    let b = password.as_bytes();
    &b[..b.len().min(72)]
}

pub fn hash_password(password: &str) -> Result<String, StoreError> {
    Ok(bcrypt::hash(pw72(password), BCRYPT_COST)?)
}

pub fn verify_password(password: &str, hashed: &str) -> bool {
    bcrypt::verify(pw72(password), hashed).unwrap_or(false)
}

pub struct UserStore {
    conn: Mutex<Connection>,
    dummy_hash: String,
}

fn row_to_user(r: &Row) -> rusqlite::Result<User> {
    Ok(User {
        id: r.get("id")?,
        username: r.get("username")?,
        role: r.get("role")?,
        active: r.get::<_, i64>("active")? != 0,
        token_version: r.get("token_version")?,
        created_at: r.get("created_at")?,
    })
}

impl UserStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| StoreError::Invalid(format!("cannot create {}: {e}", dir.display())))?;
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(10))?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS users (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 username TEXT NOT NULL UNIQUE COLLATE NOCASE,
                 password_hash TEXT NOT NULL,
                 role TEXT NOT NULL CHECK (role IN ('admin','user')),
                 active INTEGER NOT NULL DEFAULT 1,
                 token_version INTEGER NOT NULL DEFAULT 0,
                 created_at TEXT NOT NULL DEFAULT (datetime('now'))
             );
             CREATE TABLE IF NOT EXISTS search_log (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                 query TEXT NOT NULL,
                 mode TEXT NOT NULL,
                 results INTEGER NOT NULL,
                 created_at TEXT NOT NULL DEFAULT (datetime('now'))
             );
             CREATE INDEX IF NOT EXISTS ix_search_log_user ON search_log(user_id, id DESC);",
        )?;
        Ok(Self { conn: Mutex::new(conn), dummy_hash: hash_password("timing-equaliser")? })
    }

    fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    // --- queries -------------------------------------------------------------
    pub fn count(&self) -> Result<i64, StoreError> {
        Ok(self.db().query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?)
    }

    pub fn get(&self, id: i64) -> Result<Option<User>, StoreError> {
        Ok(self.db().query_row("SELECT * FROM users WHERE id = ?1", [id], row_to_user).optional()?)
    }

    pub fn list(&self) -> Result<Vec<User>, StoreError> {
        let db = self.db();
        let mut stmt = db.prepare("SELECT * FROM users ORDER BY username")?;
        let users = stmt.query_map([], row_to_user)?.collect::<Result<Vec<_>, _>>()?;
        Ok(users)
    }

    /// Checks a username and password. Slow on purpose (bcrypt): call from a blocking thread.
    pub fn authenticate(&self, username: &str, password: &str) -> Result<Option<User>, StoreError> {
        let found = self
            .db()
            .query_row("SELECT *, password_hash FROM users WHERE username = ?1", [username], |r| {
                Ok((row_to_user(r)?, r.get::<_, String>("password_hash")?))
            })
            .optional()?;
        match found {
            None => {
                verify_password(password, &self.dummy_hash); // same time as a real check
                Ok(None)
            }
            Some((user, hash)) => Ok((verify_password(password, &hash) && user.active).then_some(user)),
        }
    }

    pub fn check_password(&self, id: i64, password: &str) -> Result<bool, StoreError> {
        let hash: Option<String> =
            self.db().query_row("SELECT password_hash FROM users WHERE id = ?1", [id], |r| r.get(0)).optional()?;
        Ok(hash.is_some_and(|h| verify_password(password, &h)))
    }

    pub fn count_active_admins(&self) -> Result<i64, StoreError> {
        Ok(self.db().query_row("SELECT COUNT(*) FROM users WHERE role = 'admin' AND active = 1", [], |r| r.get(0))?)
    }

    // --- changes -------------------------------------------------------------
    pub fn create(&self, username: &str, password: &str, role: &str) -> Result<User, StoreError> {
        if !ROLES.contains(&role) {
            return Err(StoreError::Invalid("invalid role".into()));
        }
        let hash = hash_password(password)?;
        let db = self.db();
        match db.execute("INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, ?3)", params![username, hash, role]) {
            Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == rusqlite::ErrorCode::ConstraintViolation => {
                return Err(StoreError::Invalid("username already exists".into()))
            }
            other => {
                other?;
            }
        }
        let id = db.last_insert_rowid();
        Ok(db.query_row("SELECT * FROM users WHERE id = ?1", [id], row_to_user)?)
    }

    /// Bumping token_version signs the user out everywhere.
    pub fn set_password(&self, id: i64, password: &str) -> Result<(), StoreError> {
        let hash = hash_password(password)?;
        self.db().execute(
            "UPDATE users SET password_hash = ?1, token_version = token_version + 1 WHERE id = ?2",
            params![hash, id],
        )?;
        Ok(())
    }

    pub fn update(&self, id: i64, role: Option<&str>, active: Option<bool>) -> Result<Option<User>, StoreError> {
        {
            let db = self.db();
            if let Some(role) = role {
                if !ROLES.contains(&role) {
                    return Err(StoreError::Invalid("invalid role".into()));
                }
                db.execute("UPDATE users SET role = ?1, token_version = token_version + 1 WHERE id = ?2", params![role, id])?;
            }
            if let Some(active) = active {
                db.execute(
                    "UPDATE users SET active = ?1, token_version = token_version + 1 WHERE id = ?2",
                    params![active as i64, id],
                )?;
            }
        }
        self.get(id)
    }

    pub fn delete(&self, id: i64) -> Result<bool, StoreError> {
        let db = self.db();
        db.execute("DELETE FROM search_log WHERE user_id = ?1", [id])?;
        Ok(db.execute("DELETE FROM users WHERE id = ?1", [id])? > 0)
    }

    // --- search history --------------------------------------------------------
    pub fn log_search(&self, user_id: i64, query: &str, mode: &str, results: i64) -> Result<(), StoreError> {
        let q: String = query.chars().take(500).collect();
        let db = self.db();
        db.execute("INSERT INTO search_log (user_id, query, mode, results) VALUES (?1, ?2, ?3, ?4)", params![user_id, q, mode, results])?;
        // keep the table small: last 200 searches per user
        db.execute(
            "DELETE FROM search_log WHERE user_id = ?1 AND id NOT IN \
             (SELECT id FROM search_log WHERE user_id = ?1 ORDER BY id DESC LIMIT 200)",
            [user_id],
        )?;
        Ok(())
    }

    pub fn history(&self, user_id: i64, limit: i64) -> Result<Vec<HistoryItem>, StoreError> {
        let db = self.db();
        let mut stmt = db.prepare(
            "SELECT query, mode, results, MAX(created_at) AS created_at FROM search_log WHERE user_id = ?1 \
             GROUP BY query ORDER BY MAX(id) DESC LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![user_id, limit], |r| {
                Ok(HistoryItem { query: r.get(0)?, mode: r.get(1)?, results: r.get(2)?, created_at: r.get(3)? })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}
