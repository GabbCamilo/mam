# Tasks — implement-modular-extractors-and-cache

## 1. Extratores Modulares (`backend/source-integration/src/extractors/`)

- [x] 1.1 Criar `mod.rs`, `mangadex.rs`, `mangafire.rs` e `generico.rs` com testes unitários de seleção e ordenação
- [x] 1.2 Expor `extractors` em `backend/source-integration/src/lib.rs`

## 2. Cache L1 LRU no Leitor e Comandos Tauri

- [x] 2.1 Integrar `CacheL1Lru` em `reader_obter_pagina` e expor `source_listar_extratores` / `source_importar_extrator` em `backend/interface/` e `backend/src-tauri/`
- [x] 2.2 Adicionar seção **Importar via Extrator Modular** em `backend/frontend/src/api.ts` e `backend/frontend/src/App.tsx`

## 3. Validação e Empacotamento

- [x] 3.1 Executar `cargo test --workspace`, `npm run build` e `openspec validate implement-modular-extractors-and-cache`
- [x] 3.2 Regerar `dist/MAM-windows-x86_64.zip` com `build-portable-zip.ps1`
