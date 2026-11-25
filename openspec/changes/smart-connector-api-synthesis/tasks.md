# Tasks: Síntese Interna de APIs de Conectores (`smart-connector-api-synthesis`)

- [x] **MAM-SYN-001 — Schema `ApiBuscaConfig` e `ExecutorApiConector` em Rust**
  - **Arquivos:** `backend/source-integration/src/connectors.rs`, `backend/source-integration/src/api_executor.rs`
  - **Critério:** Suporte a `api_busca` em `ConectorSiteConfig` (retrocompatível), mapeamento genérico de JSON por caminho de campo, e testes unitários com payloads sintéticos.
- [x] **MAM-SYN-002 — Sintetizador de APIs no `ai-engine` (`api_synthesizer.py`)**
  - **Arquivos:** `ai-engine/mam_ai_engine/api_synthesizer.py`, `ai-engine/mam_ai_engine/pipeline.py`, `ai-engine/mam_ai_engine/server.py`, `ai-engine/tests/test_api_synthesizer.py`
  - **Critério:** Comando JSON-RPC `synthesize_search_api` detecta arrays de obras em respostas XHR/JSON, infere `endpoint`, `caminho_itens`, `campo_titulo`, `campo_id_ou_url`, `campo_capa`, com testes unitários.
- [x] **MAM-SYN-003 — Integração no Menu de Busca Interno + Auto-Persistência**
  - **Arquivos:** `backend/interface/src/lib.rs`, `backend/source-integration/src/universal.rs`
  - **Critério:** `source_buscar_obras_por_conector` prioriza `ExecutorApiConector`, aciona `synthesize_search_api` sob demanda quando não há `api_busca`, persiste em `sites.json` e alimenta `ai_memory.json`.
- [x] **MAM-SYN-004 — Observador Passivo de API na Janela Assistida (`src-tauri` + UI)**
  - **Arquivos:** `backend/src-tauri/src/lib.rs`, `backend/frontend/src/App.tsx`
  - **Critério:** Janela Assistida captura metadados de requisições JSON/XHR observadas e permite promover/testar a API sintetizada na UI de Conectores.
- [x] **MAM-SYN-005 — Verificação Final & Recompilação Portátil**
  - **Critério:** `python -m unittest`, `cargo test --workspace` (ou crates afetadas), `npm run build` e `.\packaging\windows\build-portable-zip.ps1` verdes.
