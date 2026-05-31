use std::path::Path;
use std::sync::{Mutex, Once};

use rusqlite::Connection;
use tracing::{info, warn};

use crate::error::Result;

pub mod repo;

const SCHEMA_SQL: &str = include_str!("schema.sql");
const CURRENT_USER_VERSION: i32 = 1;

static VEC_INIT: Once = Once::new();

/// sqlite-vec를 auto-extension으로 한 번만 등록한다.
/// 새 Connection을 열 때마다 자동으로 vec0 가상 테이블을 쓸 수 있게 된다.
fn ensure_vec_registered() {
    VEC_INIT.call_once(|| {
        unsafe {
            let r = rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
            if r != rusqlite::ffi::SQLITE_OK {
                warn!(code = r, "failed to register sqlite-vec as auto extension");
            } else {
                info!("sqlite-vec registered as auto extension");
            }
        }
    });
}

/// 단순한 single-connection 래퍼. CPU-bound이므로 호출자가 `spawn_blocking`으로 감싼다.
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        ensure_vec_registered();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::configure_connection(&conn)?;
        Self::migrate(&conn)?;
        info!(path = %path.display(), "sqlite opened");
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn configure_connection(conn: &Connection) -> Result<()> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Ok(())
    }

    fn migrate(conn: &Connection) -> Result<()> {
        let user_version: i32 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if user_version >= CURRENT_USER_VERSION {
            return Ok(());
        }
        info!(from = user_version, to = CURRENT_USER_VERSION, "running migrations");
        conn.execute_batch(SCHEMA_SQL)?;

        // chunk_vectors는 sqlite-vec가 로드된 경우에만 동작.
        // 로딩 실패 시 이번 세션 범위(Stage 1~2)에서는 클러스터링을 안 쓰므로 무시.
        if let Err(e) = conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS chunk_vectors USING vec0(\
                chunk_id INTEGER PRIMARY KEY,\
                embedding float[1024]\
             );",
        ) {
            warn!(?e, "chunk_vectors virtual table not created; vector search disabled");
        }

        conn.pragma_update(None, "user_version", CURRENT_USER_VERSION)?;
        Ok(())
    }

    pub fn with<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let guard = self
            .conn
            .lock()
            .map_err(|_| crate::error::AppError::Other("db mutex poisoned".into()))?;
        f(&guard)
    }

    pub fn with_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T>,
    {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| crate::error::AppError::Other("db mutex poisoned".into()))?;
        f(&mut guard)
    }
}
