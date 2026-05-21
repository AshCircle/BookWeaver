use std::sync::Arc;

mod ai;
mod commands;
mod config;
mod db;
mod error;
mod pipeline;
mod scrape;
mod search;
mod state;

use crate::ai::build_provider;
use crate::db::Db;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let cfg = config::load().unwrap_or_default();
    let db_path = config::data_dir().expect("data dir").join("bookweaver.sqlite");
    let db = Db::open(&db_path).expect("open sqlite database");
    let ai = build_provider(&cfg).map(Arc::from);
    let app_state = AppState::new(db, cfg, ai);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::start_book,
            commands::list_books,
            commands::get_book,
            commands::get_sources,
            commands::get_chapters,
            commands::get_chapter_sources,
            commands::get_settings,
            commands::update_settings,
            commands::test_provider,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = fmt().with_env_filter(filter).try_init();
}
