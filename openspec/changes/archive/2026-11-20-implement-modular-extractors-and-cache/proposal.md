# Proposal — implement-modular-extractors-and-cache

## Why

O usuário precisa importar capítulos diretamente por link/HTML/lista ordenada de URLs com suporte modular por site (MangaDex, MangaFire e espaço extensível para outros sites), além de contar com `CacheL1Lru` ativo no leitor CBZ e o pacote portátil `dist/MAM-windows-x86_64.zip` atualizado.

## What Changes

- `backend/source-integration/src/extractors/`: criar arquitetura modular com trait `ExtratorSite`, `RegistroExtratores`, `MangaDexExtractor`, `MangaFireExtractor`, `GenericoExtractor` e pipeline de empacotamento sequencial em `.cbz` com SHA-256 e registro no SQLite.
- `backend/interface/` e `backend/src-tauri/`: expor `source_listar_extratores`, `source_importar_extrator` e integrar `CacheL1Lru` (`CachePaginas`) em `reader_obter_pagina`.
- `backend/frontend/src/`: adicionar painel **Importar via Extrator Modular** na aba **Fontes & Importação**.
- `dist/MAM-windows-x86_64.zip`: regerar pacote portátil Windows com todas as alterações compiladas.

## Capabilities

### New Capabilities

- `modular-extractors-and-cache`: extratores modulares por site (MangaDex, MangaFire, genérico) para importação em `.cbz` + cache L1 LRU ativo no leitor.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos criados/modificados: `backend/source-integration/`, `backend/interface/`, `backend/src-tauri/`, `backend/frontend/src/`.
- Sem contorno de anti-bot, DRM ou paywall.
