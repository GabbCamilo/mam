# Proposal — resolve-mvp-pendencies

## Why

O levantamento técnico do projeto identificou pendências concretas para completar o MVP com qualidade: consulta real de espaço livre e detecção de disponibilidade da unidade em `mam-storage`, cache L1 LRU para páginas do leitor, persistência e revisão manual de resultados de IA (`resultados_ia` no SQLite + UI no `CbzReader`), descoberta de runtime Python portátil (`<exe_dir>/python/python.exe`) no `mam-ai-engine` / `build-portable-zip.ps1`, e atualização dos checkboxes arquivados de `2026-11-18-implement-source-integration`.

## What Changes

- `backend/storage/src/lib.rs` e `backend/storage/src/cache.rs`: consulta real de espaço livre em disco (Windows/Unix), verificação de disponibilidade da unidade (`unidade_disponivel`) e `CacheL1Lru` em memória.
- `backend/database/src/operations.rs`: operações CRUD para `resultados_ia` (`salvar_resultado_ia`, `listar_resultados_ia_capitulo`, `revisar_resultado_ia`).
- `backend/ai-engine/src/lib.rs`: descoberta automática de runtime Python embutido (`MAM_PYTHON_EXE`, `<exe_dir>/python/python.exe`, fallback `python`).
- `backend/interface/src/lib.rs`, `backend/src-tauri/src/lib.rs` e `backend/frontend/src/`: comandos Tauri de revisão de IA e painel opcional de camada IA no `CbzReader`.
- `packaging/windows/build-portable-zip.ps1`: suporte opcional a empacotamento de runtime Python embeddable (`-PythonRuntimeDir`).
- `openspec/changes/archive/2026-11-18-implement-source-integration/tasks.md`: atualização dos checkboxes `[x]` refletindo o código já implementado.

## Capabilities

### New Capabilities

- `mvp-pendencies`: cache L1 LRU, espaço real em disco, detecção de unidade, persistência/revisão de resultados de IA e suporte a runtime Python portátil.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos criados/modificados: `backend/storage/`, `backend/database/`, `backend/ai-engine/`, `backend/interface/`, `backend/src-tauri/`, `backend/frontend/src/`, `packaging/windows/`, `openspec/changes/archive/2026-11-18-implement-source-integration/tasks.md`.
