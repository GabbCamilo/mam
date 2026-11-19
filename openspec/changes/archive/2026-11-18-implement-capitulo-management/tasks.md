# Tasks

## 1. Estrutura do crate mam-chapter

- [x] 1.1 Adicionar `mam-chapter` ao `backend/Cargo.toml` workspace members
- [x] 1.2 Criar `backend/chapter/Cargo.toml` com dependências (walkdir, zip, sha2, mam-database, mam-core, mam-storage, thiserror, serde, serde_json, tokio)
- [x] 1.3 Criar `backend/chapter/src/lib.rs` (ChapterManager ponto de entrada)
- [x] 1.4 Criar `backend/chapter/src/detector.rs` (varredura walkdir de .cbz)
- [x] 1.5 Criar `backend/chapter/src/parser.rs` (leitura de páginas do CBZ)
- [x] 1.6 Criar `backend/chapter/src/integrity.rs` (hash SHA-256)
- [x] 1.7 Criar `backend/chapter/src/operations.rs` (registro no banco)

## 2. Implementar mam-storage

- [x] 2.1 Verificar `backend/storage/Cargo.toml` (dependências sha2, walkdir, tokio, mam-core)
- [x] 2.2 Implementar `backend/storage/src/lib.rs` (StorageManager: identificar bibliotecas, unidades, espaço)

## 3. Comandos Tauri para capítulos

- [x] 3.1 Comando `chapter_detectar` (detectar capítulos na biblioteca)
- [x] 3.2 Comando `chapter_listar_por_obra` (listar capítulos de uma obra)
- [x] 3.3 Comando `library_raiz` (caminho da raiz da biblioteca)
- [x] 3.4 Comandos existentes: `db_health`, `db_list_obras`, `health_check`
- [x] 3.5 Registrar comandos no `invoke_handler` do src-tauri

## 4. Validação e teste

- [x] 4.1 Teste: detecção de .cbz em diretório temporário (detector::tests)
- [x] 4.2 Teste: parsing de páginas de CBZ (parser::tests)
- [x] 4.3 Teste: cálculo de hash SHA-256 (integrity::tests)
- [x] 4.4 `cargo build` e `cargo test` passam sem erros (todos os crates)
