# Tasks

## 1. Estrutura do crate mam-archive

- [x] 1.1 Adicionar `mam-archive` ao `backend/Cargo.toml` workspace members
- [x] 1.2 Criar `backend/archive/Cargo.toml` com dependências (flate2, zip, walkdir, sha2, mam-core, mam-database, mam-storage, thiserror, serde, serde_json, tokio)
- [x] 1.3 Criar `backend/archive/src/lib.rs` (ArchiveManager ponto de entrada)
- [x] 1.4 Criar `backend/archive/src/criacao.rs` (geração de CBZ)
- [x] 1.5 Criar `backend/archive/src/leitura.rs` (leitura e validação de CBZ)
- [x] 1.6 Criar `backend/archive/src/validacao.rs` (detecção de páginas ausentes, integridade)
- [x] 1.7 Criar `backend/archive/src/operacoes_seguras.rs` (escrita com temp + rename)

## 2. Comandos Tauri para CBZ

- [x] 2.1 Comando `archive_criar_cbz` (criar CBZ a partir de diretório de imagens)
- [x] 2.2 Comando `archive_ler_cbz` (ler páginas de um CBZ existente)
- [x] 2.3 Comando `archive_validar_cbz` (validar integridade de CBZ)
- [x] 2.4 Registrar comandos no `invoke_handler` do src-tauri

## 3. Validação e teste

- [x] 3.1 Criar teste: geração de CBZ a partir de imagens temporárias (criacao::tests)
- [x] 3.2 Criar teste: leitura de páginas de CBZ existente (leitura::tests)
- [x] 3.3 Criar teste: validação de CBZ corrompido (validacao::tests)
- [x] 3.4 Criar teste: detecção de páginas ausentes (leitura::tests)
- [x] 3.5 Teste: escrita atômica temp+rename (operacoes_seguras::tests)
- [x] 3.6 `cargo build` e `cargo test` passam sem erros (todos os crates)
