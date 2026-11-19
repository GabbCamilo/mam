use std::sync::Arc;
use std::path::PathBuf;

use mam_database::Database;
use mam_interface::{db_list_obras, db_health};

#[tauri::command]
fn health_check() -> String {
    "ok".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      health_check,
      db_health,
      db_list_obras,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Inicializa o banco de dados SQLite como estado gerenciado pelo Tauri
      let app_handle = app.handle();
      let caminho_db = app_handle.path_resolver().app_data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("database.db");

      let raiz_biblioteca = app_handle.path_resolver().app_data_dir()
        .unwrap_or_else(|| PathBuf::from("."));

      // Abre o banco de dados SQLite (runtime do Tauri executa setup de forma síncrona)
      let db = tokio::runtime::Runtime::new().unwrap().block_on(async {
        Database::abrir(caminho_db, raiz_biblioteca).await
          .expect("Falha ao inicializar banco de dados SQLite")
      });

      app.manage(Arc::new(db));

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
