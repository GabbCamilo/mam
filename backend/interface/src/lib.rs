//! # mam-interface
//! Comandos e eventos Tauri para comunicação com o frontend (React + TypeScript).
//!
//! Depende de `mam-core` para tipos de identificador e tarefa.

use tauri::command;

#[command]
pub fn health_check() -> String {
    "ok".to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
