# Tasks: Assisted Learning, Auto-Capture & Single Chapter Target

- [x] **MAM-AL-001 — Memória Progressiva Cross-Site (`ai_memory.rs`)**
  - **Arquivos:** `backend/source-integration/src/ai_memory.rs`, `backend/source-integration/src/lib.rs`
  - **Critério:** Persistência atômica em `System/connectors/ai_memory.json`, reforço por `hits`/`misses`, consulta ranqueada e testes unitários.
- [x] **MAM-AL-002 — Aprendizado Profundo DOM + URL (`connectors.rs` e `extractors/mod.rs`)**
  - **Arquivos:** `backend/source-integration/src/connectors.rs`, `backend/source-integration/src/extractors/mod.rs`
  - **Critério:** `aprender_de_sessao` infere `padroes_url_capitulo`, `seletor_link` e `seletores_imagem`, integra com `ai_memory.rs` e suporta `obra_id_destino` em `EntradaImportacao`.
- [x] **MAM-AL-003 — Auto-Captura, Aprendizado Passivo e Eventos na Janela Assistida (`src-tauri/src/lib.rs`)**
  - **Arquivos:** `backend/src-tauri/src/lib.rs`
  - **Critério:** `on_page_load` na janela `mam-assistido` captura quando a página estabiliza fora de desafio humano e emite `assisted-window://captured`; comandos de aprendizado usam sessão obra+capítulo.
- [x] **MAM-AL-004 — UI: Auto-Importação na Janela Assistida e Seletor de Destino no Capítulo Único (`api.ts`, `App.tsx`)**
  - **Arquivos:** `backend/frontend/src/api.ts`, `backend/frontend/src/App.tsx`
  - **Critério:** Toggle de auto-captura/auto-importação funcional via evento Tauri e seletor de destino (`auto` / `existente` / `novo`) no modo Capítulo Único.
- [x] **MAM-AL-005 — Verificação Final**
  - **Critério:** `cargo test --workspace` (ou crates afetadas), `npm run build` em `backend/frontend/` e `python -m unittest discover -s ai-engine/tests -v` verdes.
