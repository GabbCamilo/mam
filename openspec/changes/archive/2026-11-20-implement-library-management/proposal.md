# Proposal — implement-library-management

## Why

O MAM já possui persistência SQLite (`mam-database`), gerenciamento de capítulos (`mam-chapter`), storage (`mam-storage`), criação/leitura de CBZ (`mam-archive`) e integração com fontes (`mam-source-integration`), porém o crate `mam-library` ainda é apenas um esqueleto. É necessário implementar o **Library Manager** para consolidar o catálogo de obras, volumes e capítulos, gerenciar metadados, suportar organização híbrida de volumes e configurar caminhos relativos à raiz da biblioteca portátil.

## What Changes

- `backend/library/Cargo.toml`: adicionar dependências necessárias (`mam-database`, `mam-storage`, `tokio`).
- `backend/library/src/lib.rs`: implementar `LibraryManager` como ponto de entrada do domínio de biblioteca.
- `backend/library/src/catalogo.rs`: operações de catálogo para obras, volumes e capítulos com organização híbrida (capítulos vinculados a volume ou diretamente à obra).
- `backend/library/src/metadados.rs`: leitura, atualização e validação de metadados complementares em JSON.
- `backend/interface/src/lib.rs` e `backend/src-tauri/src/lib.rs`: expor e registrar comandos Tauri para gerenciamento de biblioteca (`library_criar_obra`, `library_listar_obras`, `library_obter_detalhes_obra`, `library_criar_volume`, `library_atualizar_metadados`).

## Capabilities

### New Capabilities

- `library-management`: gerenciamento do catálogo de obras, volumes e capítulos, organização híbrida de volumes, metadados complementares e resolução de caminhos relativos à raiz da biblioteca.

### Modified Capabilities

- (nenhuma — nova capacidade integrada aos comandos Tauri existentes)

## Impact

- Arquivos criados/modificados no workspace Rust: `backend/library/`, `backend/interface/`, `backend/src-tauri/`.
- Sem alterações destrutivas em CBZ originais ou exclusão automática de arquivos da origem.
- Todos os caminhos persistidos permanecem relativos à raiz da biblioteca.
