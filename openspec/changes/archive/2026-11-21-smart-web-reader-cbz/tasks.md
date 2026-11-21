## 1. AI Engine (Python)
- [x] 1.1 Criar `ai-engine/mam_ai_engine/web_reader.py` com cliente CDP WebSocket em stdlib pura (descoberta multi-página/multi-idioma e extração completa de páginas de capítulo)
- [x] 1.2 Adicionar `smart_discover_chapters`, `smart_extract_chapter_pages` e `analyze_reader_dom` em `ai-engine/mam_ai_engine/pipeline.py` e registrar em `server.py`
- [x] 1.3 Adicionar testes unitários em `ai-engine/tests/`

## 2. Backend Rust (`source-integration`, `interface`, `src-tauri`)
- [x] 2.1 Corrigir `eh_url_imagem_valida` em `backend/source-integration/src/extractors/mod.rs` para rejeitar `.js`/`.css`/logos/ícones
- [x] 2.2 Integrar `AiEngineManager` (`smart_discover_chapters` / `smart_extract_chapter_pages`) com `idioma_preferido` em `mod.rs`, `mangafire.rs` e `generico.rs`
- [x] 2.3 Atualizar `source_descobrir_por_link` em `backend/interface/src/lib.rs` e `backend/src-tauri/src/lib.rs` para aceitar `idioma_preferido`

## 3. Frontend & Verificação
- [x] 3.1 Adicionar seletor de idioma alvo (`PT-BR` padrão, `ENG`, `JPN`, `Todos`) no modal de importação em `backend/frontend/src/App.tsx`
- [x] 3.2 Rodar `openspec validate`, testes Python, `npm run build` e `cargo test --workspace`
- [x] 3.3 Arquivar mudança OpenSpec, enviar commit para `origin/main` e gerar pacote portátil Windows preservando `MAM_LIBRARY`
