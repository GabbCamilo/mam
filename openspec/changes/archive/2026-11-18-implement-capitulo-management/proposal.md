# Proposal — implement-capitulo-management

## Why

O MAM necessita detectar capítulos a partir de arquivos .cbz armazenados na biblioteca, identificar páginas numeradas e ordenadas, calcular integridade (SHA-256), e registrar esses dados no banco SQLite. O capítulo é a unidade fundamental de leitura e arquivamento, e sua detecção automática é pré-requisito para o leitor integrado, transferência e sincronização.

## What Changes

- `backend/chapter/Cargo.toml`: adicionar dependências (walkdir, flate2, zip, sha2, mam-database, mam-core, mam-storage)
- `backend/chapter/src/lib.rs`: ponto de entrada, estrutura ChapterManager
- `backend/chapter/src/detector.rs`: detecção de arquivos .cbz no filesystem da biblioteca
- `backend/chapter/src/parser.rs`: leitura do CBZ (lista de páginas, ordenação, quantidade)
- `backend/chapter/src/integrity.rs`: cálculo de hash SHA-256 do arquivo .cbz
- `backend/chapter/src/operations.rs`: operações de registro/consulta de capítulos no banco
- `backend/storage/Cargo.toml`: adicionar dependências ausentes (sha2, walkdir) já declaradas
- `backend/storage/src/lib.rs`: implementar StorageManager (identificação de bibliotecas, unidades, espaço)
- `backend/interface/src/lib.rs`: comandos Tauri para listar capítulos, verificar integridade
- `backend/src-tauri/src/lib.rs`: registrar novos comandos no invoke_handler

## Capabilities

### New Capabilities

- `chapter-detection`: detecção automática de arquivos .cbz na estrutura Obra > Volume > Capítulo
- `chapter-parsing`: leitura de páginas de um CBZ (quantidade, ordem, nomes)
- `chapter-integrity`: validação SHA-256 de arquivos de capítulo
- `chapter-registration`: registro de capítulos no banco de dados SQLite
- `storage-management`: identificação de bibliotecas MAM e unidades de armazenamento

### Modified Capabilities

- `mam-interface`: novos comandos Tauri (listar capítulos, verificar integridade)

## Impact

- Arquivos criados: `backend/chapter/` (Cargo.toml, src/*), `backend/storage/src/lib.rs`
- Arquivos modificados: `backend/interface/src/lib.rs`, `backend/src-tauri/src/lib.rs`, `backend/storage/Cargo.toml` (se necessário)
- Nenhuma dependência nova instalada além de crates já declaradas (walkdir, flate2, zip, sha2)

## Regras respeitadas

- Caminhos relativos à raiz da biblioteca
- SHA-256 para validação de integridade
- Transações curtas no banco de dados
- Nunca excluir arquivos da origem
- Detecção só de leitura (não modifica CBZ originais)
