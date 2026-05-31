use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ai::AiProvider;
use crate::config::UserConfig;
use crate::db::Db;

/// 앱 전역 상태. Tauri의 `manage`를 통해 등록한다.
pub struct AppState {
    pub db: Arc<Db>,
    pub ai: Mutex<Option<Arc<dyn AiProvider>>>,
    pub config: Mutex<UserConfig>,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(db: Db, config: UserConfig, ai: Option<Arc<dyn AiProvider>>) -> Self {
        Self {
            db: Arc::new(db),
            ai: Mutex::new(ai),
            config: Mutex::new(config),
            http: reqwest::Client::builder()
                .user_agent("BookWeaver/0.1 (+local)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client build"),
        }
    }
}
