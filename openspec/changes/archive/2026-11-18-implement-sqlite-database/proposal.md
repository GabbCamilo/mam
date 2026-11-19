# Proposal — implement-sqlite-database

## Why

O MAM necessita de persistência local para obras, volumes, capítulos, metadados, configurações, histórico, estado de integridade e referências a resultados de IA. O banco de dados deve ser SQLite (arquivo único, sem servidor externo), compatível com portabilidade entre computadores e unidades externas, armazenando apenas referências relativas.

## What Changes

- `backend/Cargo.toml`: adicionar crate `mam-database` ao workspace
- `backend/database/Cargo.toml`: novo crate com dependências rusqlite, serde, uuid, chrono
- `backend/database/src/lib.rs`: módulo de banco de dados (conexão, migrações, schema)
- `backend/database/src/schema.rs`: definição das tabelas (obras, volumes, capítulos, metadados, config, histórico, integridade)
- `backend/database/src/migrations.rs`: sistema de migrações versionadas
- `backend/database/src/models.rs`: structs de modelo (Obra, Volume, Capítulo, etc.)
- `backend/database/src/operations.rs`: operações CRUD controladas com transações curtas
- `backend/core/src/lib.rs`: exportar tipos de domínio compartilhados (Identificador, CaminhoRelativo)
- `backend/interface/src/lib.rs`: comandos Tauri para operações de banco (ex.: health_check já existente)

## Capabilities

### New Capabilities

- `sqlite-database`: persistência local SQLite com schema de biblioteca (obra > volume > capítulo)
- `database-migrations`: sistema de migrações versionadas para evolução do schema
- `database-operations`: operações CRUD com transações curtas e controle de concorrência

### Modified Capabilities

- `mam-core`: adicionar tipos de domínio compartilhados (UUID, caminhos relativos)

## Impact

- Arquivos criados: `backend/database/` (Cargo.toml, src/*), modificações em `backend/Cargo.toml`, `backend/core/src/lib.rs`
- Nenhuma dependência nova instalada além de `rusqlite` (via cargo build)
- Sem impacto no frontend (navegação continua via comandos Tauri existentes)

## Regras respeitadas

- SQLite como banco local, sem instalação externa
- Referências relativas à raiz da biblioteca (não caminhos absolutos)
- Transações curtas e operações controladas
- Não permitir duas instâncias escrevendo simultaneamente sem mecanismo explícito
- WAL avaliado com atenção à portabilidade
- Toda documentação em português brasileiro
- Não criar ADRs nesta fase
