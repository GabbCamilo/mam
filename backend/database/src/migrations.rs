//! Sistema de migrações versionadas para o banco de dados MAM.
//! As migrações são arquivos SQL numerados, aplicados em ordem na primeira inicialização.
//! O controle de versão é armazenado na tabela schema_migrations.

use rusqlite::Connection;
use std::path::Path;

/// Lista de migrações em ordem de aplicação.
/// Cada tupla: (número da versão, SQL da migração).
const MIGRACOES: &[(i32, &str)] = &[
    (
        1,
        include_str!("migrations/001_initial.sql"),
    ),
    (
        2,
        include_str!("migrations/002_capitulos_idioma.sql"),
    ),
];

/// Executa todas as migrações pendentes.
/// Cria a tabela schema_migrations se não existir e aplica migrações não executadas.
pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    // Cria tabela de controle de migrações
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            aplicado_em TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Obtém versões já aplicadas
    let versoes_aplicadas: Vec<i32> = conn
        .prepare("SELECT version FROM schema_migrations")?
        .query_map([], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();

    // Aplica migrações pendentes em ordem
    for (versao, sql) in MIGRACOES {
        if !versoes_aplicadas.contains(versao) {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                [versao],
            )?;
        }
    }

    Ok(())
}

/// Caminho padrão para o arquivo de migração inicial (se necessário carregar de arquivo externo).
pub fn caminho_migracao_inicial() -> &'static Path {
    Path::new("migrations/001_initial.sql")
}
