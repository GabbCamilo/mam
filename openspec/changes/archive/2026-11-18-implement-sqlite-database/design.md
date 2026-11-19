# Design — implement-sqlite-database

## Context

O MAM utiliza SQLite como banco de dados local (decisão aprovada na fundação técnica). O banco deve residir dentro da estrutura `System/database.db` da biblioteca, armazenar referências relativas aos arquivos .cbz, e suportar operação offline. O backend Rust (workspace modular) gerencia a conexão e expõe operações via comandos Tauri para o frontend.

## Goals / Non-Goals

**Goals:**
- Criar crate `mam-database` com conexão SQLite (rusqlite)
- Definir schema: obras, volumes, capítulos, metadados, configurações, histórico, integridade
- Sistema de migrações versionadas
- Operações CRUD com transações curtas
- Controle de concorrência (uma instância escrevendo por vez)
- Caminhos relativos à raiz da biblioteca
- Comandos Tauri para expor operações ao frontend

**Non-Goals:**
- Servidor de banco remoto (PostgreSQL, MySQL, etc.)
- Sincronização automática de banco entre unidades (foi especificada separadamente)
- Criptografia do banco (SQLCipher) — avaliar posteriormente
- ORM completo (Sequelize/Diesel) — usar rusqlite diretamente para simplicidade e controle

## Decisions

1. **Biblioteca: rusqlite** — binding síncrono e maduro para SQLite, não requer runtime assíncrono, adequado a operações locais curtas. Alternativa `sqlx` rejeitada por exigir runtime async e driver adicional para SQLite.
2. **Migrações: SQL puro versionado** — arquivos `.sql` numerados (001_initial.sql, 002_..., etc.) executados em ordem na primeira inicialização. Controle de versão em tabela `schema_migrations`.
3. **Conexão: uma instância por biblioteca** — o backend abre uma conexão SQLite por biblioteca aberta, com `PRAGMA journal_mode=WAL` avaliado (portabilidade entre SO).
4. **Caminhos relativos: `std::path::PathBuf` relativos à raiz** — armazenados como `String` no banco, resolvidos em tempo de execução contra a raiz da biblioteca (`MAM_LIBRARY/`).
5. **Tipos de domínio em `mam-core`**: `ObraId`, `VolumeId`, `CapituloId` (wrappers de `Uuid`), `CaminhoRelativo` (wrapper de `PathBuf`/String).
6. **Controle de concorrência: mutex no nível da aplicação** — `tokio::sync::Mutex` protegendo a conexão SQLite, garantindo que apenas uma tarefa escreva por vez.
7. **Tabela `schema_migrations`**: registra versões aplicadas para evitar reaplicação.

## Arquitetura

```
backend/
├── core/                 # mam-core — tipos de domínio (ids, caminhos relativos)
├── database/             # mam-database — conexão SQLite, schema, migrações, operações
│   ├── src/
│   │   ├── lib.rs        # ponto de entrada, Database struct
│   │   ├── schema.rs     # CREATE TABLE definitions
│   │   ├── migrations.rs # sistema de migrações versionadas
│   │   ├── models.rs     # structs Obra, Volume, Capitulo, etc.
│   │   └── operations.rs # CRUD com transações curtas
├── library/              # mam-library — depende de mam-database
├── chapter/              # mam-chapter — depende de mam-database
├── storage/              # mam-storage — depende de mam-database
└── src-tauri/            # app Tauri — expõe comandos de banco via interface
```

## Esquema de banco (tabelas)

| Tabela | Propósito | Chave |
|--------|-----------|-------|
| `schema_migrations` | controle de versão do schema | `version` INTEGER |
| `obras` | catálogo de obras | `id` UUID PK |
| `volumes` | volumes de uma obra | `id` UUID PK, `obra_id` FK |
| `capitulos` | capítulos de um volume | `id` UUID PK, `volume_id` FK |
| `metadados_capitulo` | páginas, título, origem, integridade | `capitulo_id` FK |
| `configuracao_biblioteca` | configurações da biblioteca | `chave` PK |
| `historico_operacoes` | log de operações (arquivamento, transferência) | `id` UUID PK |
| `integridade_arquivo` | hash SHA-256, estado de integridade | `arquivo_caminho` PK |
| `resultados_ia` | referências a resultados de IA (JSON) | `id` UUID PK |

## Regras obrigatórias

- Caminhos de arquivo armazenados como relativos à raiz da biblioteca
- Nunca armazenar caminhos absolutos (letra de unidade)
- Transações curtas (abrir → executar → fechar/commit)
- Bloqueio de escrita exclusivo por instância de banco
- WAL mode avaliado: habilitar se não prejudicar portabilidade entre Windows/Linux

## Risks / Trade-offs

- [rusqlite síncrono] operações de banco bloqueiam a thread; mitigado com `tokio::task::spawn_blocking` para não travar o loop async do Tauri.
- [WAL portabilidade] arquivos `-wal` e `-shm` podem não ser copiados corretamente em transferência de biblioteca; mitigado com checkpoint antes de transferir.
- [Concorrência] duas instâncias do MAM apontando para o mesmo `database.db` corrompem; mitigado com mutex de aplicação + documentação de uso single-instance por biblioteca.

## Migration Plan

1. Criar crate `mam-database` no workspace
2. Adicionar `rusqlite` como dependência
3. Implementar `schema.rs`, `migrations.rs`, `models.rs`, `operations.rs`
4. Adicionar tipos de domínio em `mam-core`
5. Expor comandos Tauri em `mam-interface` / `src-tauri`
6. Validar com `cargo build` e `cargo test`
7. Arquivar change

## Open Questions

- Nome exato do arquivo do banco: `database.db` vs `library.db` (decisão: `database.db` conforme estrutura conceitual)
- Local da tabela `schema_migrations` (embutida no próprio banco)
- Estratégia de backup automático antes de migrações (avaliar)
