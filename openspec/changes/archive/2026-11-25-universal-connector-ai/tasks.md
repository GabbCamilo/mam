# Tasks: Conector Universal Gerenciado pela IA Portátil

- [x] **MAM-UNI-001 — Orquestrador `ConectorUniversal` (`backend/source-integration/src/universal.rs`)**
  - **Arquivos:** `backend/source-integration/src/universal.rs`, `backend/source-integration/src/lib.rs`
  - **Critério:** `correlacionar_link` unifica `sites.json` + `ai_memory.json` com testes unitários.
- [x] **MAM-UNI-002 — Busca Unificada `"universal"` (`backend/interface/src/lib.rs`)**
  - **Arquivos:** `backend/interface/src/lib.rs`, `backend/source-integration/src/extractors/mod.rs`
  - **Critério:** `source_buscar_obras_por_conector` suporta `conector_id == "universal"` agregando resultados em paralelo com tolerância a falhas; `extractors/mod.rs` usa `ConectorUniversal::correlacionar_link`.
- [x] **MAM-UNI-003 — Hints de Memória no `ai-engine` (`web_reader.py`, `pipeline.py`)**
  - **Arquivos:** `ai-engine/mam_ai_engine/web_reader.py`, `ai-engine/mam_ai_engine/pipeline.py`
  - **Critério:** `memory_hints` opcionais integrados em `smart_discover_chapters` e `smart_extract_chapter_pages`.
- [x] **MAM-UNI-004 — UI: Conector `Universal (IA Portátil)` (`backend/frontend/src/App.tsx`)**
  - **Arquivos:** `backend/frontend/src/App.tsx`
  - **Critério:** `universal` como padrão na busca da aba Fontes com badge de fonte de origem nos resultados.
- [x] **MAM-UNI-005 — Verificação Final**
  - **Critério:** `npm run build`, `cargo test -p mam-source-integration`, `cargo check -p app` e `python -m unittest discover -s tests -v` verdes.
