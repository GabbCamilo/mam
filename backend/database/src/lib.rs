//! # mam-database
//! Persistência local SQLite para o MAM: obras, volumes, capítulos, metadados,
//! configurações, histórico, integridade e referências a resultados de IA.
//!
//! Utiliza rusqlite (binding síncrono) com migrações versionadas e transações curtas.
//! Todos os caminhos de arquivo são armazenados como relativos à raiz da biblioteca.

use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use tokio::sync::Mutex;

mod schema;
mod migrations;
pub mod models;
mod operations;

use crate::migrations::run_migrations;

use mam_core::CaminhoRelativo;

/// Erros do módulo de banco de dados.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Erro SQL: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("Banco não inicializado")]
    NotInitialized,
    #[error("Obra não encontrada: {0}")]
    ObraNaoEncontrada(String),
    #[error("Capítulo não encontrado: {0}")]
    CapituloNaoEncontrado(String),
}

/// Wrapper assíncrono sobre a conexão SQLite.
/// O Mutex garante que apenas uma tarefa escreva por vez (controle de concorrência).
pub struct Database {
    conn: Mutex<Connection>,
    raiz_biblioteca: PathBuf,
}

impl Database {
    /// Abre (ou cria) o banco de dados em `caminho_db` e executa migrações.
    pub async fn abrir(caminho_db: PathBuf, raiz_biblioteca: PathBuf) -> Result<Self, DatabaseError> {
        let conn = Connection::open_with_flags(
            &caminho_db,
            OpenFlags::default()
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .map_err(DatabaseError::Sql)?;

        // Configurações de integridade e desempenho (otimizadas para baixo uso de RAM e I/O)
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(DatabaseError::Sql)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(DatabaseError::Sql)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(DatabaseError::Sql)?;
        conn.pragma_update(None, "busy_timeout", 5000)
            .map_err(DatabaseError::Sql)?;
        conn.pragma_update(None, "temp_store", "MEMORY")
            .map_err(DatabaseError::Sql)?;
        conn.pragma_update(None, "cache_size", -8000)
            .map_err(DatabaseError::Sql)?;

        // Executa migrações
        run_migrations(&conn).map_err(DatabaseError::Sql)?;

        Ok(Database {
            conn: Mutex::new(conn),
            raiz_biblioteca,
        })
    }

    /// Retorna a raiz da biblioteca associada a esta instância.
    pub fn raiz(&self) -> &PathBuf {
        &self.raiz_biblioteca
    }

    /// Executa uma operação dentro de uma transação curta.
    pub async fn transacao<F, T>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(&Connection) -> rusqlite::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction().map_err(DatabaseError::Sql)?;
        let resultado = f(&tx).map_err(DatabaseError::Sql)?;
        tx.commit().map_err(DatabaseError::Sql)?;
        Ok(resultado)
    }

    /// Executa checkpoint WAL truncando o arquivo auxiliar para manter o banco portátil em arquivo único.
    pub async fn checkpoint_truncar(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().await;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(DatabaseError::Sql)?;
        Ok(())
    }

    /// Caminho absoluto resolvido a partir de um caminho relativo da biblioteca.
    pub fn resolver_caminho(&self, relativo: &CaminhoRelativo) -> PathBuf {
        self.raiz_biblioteca.join(relativo.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn abre_banco_em_memoria() {
        let temp_dir = env::temp_dir().join(format!("mam_test_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("database.db");

        let db = Database::abrir(db_path, temp_dir.clone()).await;
        assert!(db.is_ok(), "deve abrir banco de dados: {:?}", db.err());

        // Limpeza
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
