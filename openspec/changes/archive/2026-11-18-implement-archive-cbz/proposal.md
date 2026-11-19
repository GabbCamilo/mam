# Proposal — implement-archive-cbz

## Why

O MAM necessita gerar e ler arquivos CBZ (formato principal dos capítulos). O Archive Manager será responsável por criar arquivos CBZ a partir de imagens, validar arquivos CBZ existentes, identificar páginas ausentes e garantir escrita segura com arquivos temporários. Esta é a etapa de arquivamento que converte conteúdo obtido de fontes autorizadas no formato padrão da biblioteca.

## What Changes

- `backend/archive/Cargo.toml`: novo crate com dependências (flate2, zip, walkdir, sha2, mam-core, mam-database, mam-storage)
- `backend/archive/src/lib.rs`: ponto de entrada, ArchiveManager
- `backend/archive/src/criacao.rs`: geração de arquivos CBZ a partir de imagens
- `backend/archive/src/leitura.rs`: leitura e validação de arquivos CBZ existentes
- `backend/archive/src/validacao.rs`: validação de integridade de CBZ, detecção de páginas ausentes
- `backend/archive/src/operacoes_seguras.rs`: escrita com arquivos temporários, operações atômicas
- `backend/interface/src/lib.rs`: comandos Tauri para criar CBZ, validar CBZ, consultar páginas
- `backend/src-tauri/src/lib.rs`: registrar novos comandos no invoke_handler
- `backend/Cargo.toml`: adicionar `mam-archive` ao workspace members

## Capabilities

### New Capabilities

- `archive-criacao-cbz`: geração de arquivos CBZ a partir de imagens sequenciais
- `archive-leitura-cbz`: leitura de arquivos CBZ existentes (páginas, metadados)
- `archive-validacao-cbz`: validação de integridade, detecção de páginas ausentes
- `archive-escrita-segura`: operações de escrita com arquivos temporários e commit atômico

### Modified Capabilities

- `mam-interface`: novos comandos Tauri (criar CBZ, validar CBZ, listar páginas)

## Impact

- Arquivos criados: `backend/archive/` (Cargo.toml, src/*)
- Arquivos modificados: `backend/Cargo.toml`, `backend/interface/src/lib.rs`, `backend/src-tauri/src/lib.rs`
- Nenhuma dependência nova instalada além de crates já declaradas (flate2, zip, walkdir, sha2)

## Regras respeitadas

- Caminhos relativos à raiz da biblioteca
- CBZ originais nunca modificados pelo processamento de IA (apenas leitura para validação)
- Geração utiliza arquivos temporários e operações de escrita seguras
- Nunca excluir arquivos da origem
- SHA-256 para validação de integridade
