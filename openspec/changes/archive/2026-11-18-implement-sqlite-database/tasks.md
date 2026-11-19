# Tasks

## 1. Estrutura do crate mam-database

- [x] 1.1 Adicionar `mam-database` ao `backend/Cargo.toml` workspace members
- [x] 1.2 Criar `backend/database/Cargo.toml` com dependências (rusqlite, serde, serde_json, uuid, chrono, thiserror, tokio, mam-core)
- [x] 1.3 Criar `backend/database/src/lib.rs` (struct Database, conexão, inicialização)
- [x] 1.4 Criar `backend/database/src/schema.rs` (CREATE TABLE para todas as tabelas)
- [x] 1.5 Criar `backend/database/src/migrations.rs` (sistema de migrações versionadas)
- [x] 1.6 Criar `backend/database/src/models.rs` (structs Obra, Volume, Capitulo, Metadados, Config, Historico, Integridade, ResultadoIA)
- [x] 1.7 Criar `backend/database/src/operations.rs` (CRUD com transações curtas)

## 2. Tipos de domínio em mam-core

- [x] 2.1 Adicionar `ObraId`, `VolumeId`, `CapituloId` (wrappers de Uuid)
- [x] 2.2 Adicionar `CaminhoRelativo` (wrapper de String para caminhos relativos)
- [x] 2.3 Adicionar `EstadoIntegridade` (enum: Valido, Invalido, Desconhecido)

## 3. Integração com workspace e build

- [x] 3.1 Verificar que `cargo build` compila todos os crates incluindo mam-database
- [x] 3.2 Verificar que `cargo test` passa em todos os crates

## 4. Comandos Tauri para banco de dados

- [x] 4.1 Adicionar comando `db_health` (verificar se banco está acessível)
- [x] 4.2 Adicionar comando `db_init` (inicializar banco e rodar migrações) — implementado no setup do Tauri
- [x] 4.3 Adicionar comando `db_list_obras` (listar obras cadastradas)
- [x] 4.4 Registrar comandos no `invoke_handler` do src-tauri

## 5. Validação e teste

- [x] 5.1 Criar teste de integração: abrir banco, criar obra, ler obra
- [x] 5.2 Criar teste de migração: verificar que schema_migrations é populado (via run_migrations)
- [x] 5.3 Verificar que caminhos relativos são armazenados corretamente
- [x] 5.4 `cargo build` e `cargo test` passam sem erros
