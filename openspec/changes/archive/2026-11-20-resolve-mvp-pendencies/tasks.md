# Tasks — resolve-mvp-pendencies

## 1. Storage Manager: espaço real, detecção de unidade e Cache L1 LRU

- [x] 1.1 Criar `backend/storage/src/cache.rs` com `CacheL1Lru` e testes unitários
- [x] 1.2 Implementar `espaco_disponivel` real e `unidade_disponivel` em `backend/storage/src/lib.rs`

## 2. Persistência e revisão de IA + Runtime Python portátil

- [x] 2.1 Adicionar `salvar_resultado_ia`, `listar_resultados_ia_capitulo` e `revisar_resultado_ia` em `backend/database/src/operations.rs`
- [x] 2.2 Atualizar `AiEngineManager` em `backend/ai-engine/src/lib.rs` com `detectar_executavel_python` (`MAM_PYTHON_EXE`, `<exe_dir>/python/python.exe`, `python`)
- [x] 2.3 Expor `ai_revisar_resultado` e `ai_listar_resultados_capitulo` em `backend/interface/src/lib.rs` e `backend/src-tauri/src/lib.rs`
- [x] 2.4 Integrar painel opcional de revisão de IA no `CbzReader` (`backend/frontend/src/api.ts` e `backend/frontend/src/components/CbzReader.tsx`)

## 3. Empacotamento e higiene OpenSpec

- [x] 3.1 Atualizar `packaging/windows/build-portable-zip.ps1` para suportar `-PythonRuntimeDir`
- [x] 3.2 Atualizar checkboxes de `openspec/changes/archive/2026-11-18-implement-source-integration/tasks.md`
- [x] 3.3 Executar `npm run build`, `cargo test --workspace` e `openspec validate resolve-mvp-pendencies`
