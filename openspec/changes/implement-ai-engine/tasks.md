# Tasks — implement-ai-engine

## 1. Processo auxiliar Python (`ai-engine/`)

- [ ] 1.1 Criar pacote `ai-engine/mam_ai_engine/` (`__init__.py`, `protocol.py`, `pipeline.py`, `models_manager.py`, `server.py`) com loop JSON-RPC 2.0 sobre stdio (JSONL)
- [ ] 1.2 Criar `ai-engine/tests/test_server.py` validando `ping`, `ocr`, `detect_bubbles`, `translate`, `models_list` e `shutdown`

## 2. Coordenador Rust (`backend/ai-engine/`)

- [ ] 2.1 Implementar tipos JSON-RPC 2.0 e `AiEngineError` em `backend/ai-engine/src/protocol.rs`
- [ ] 2.2 Implementar `AiEngineManager` em `backend/ai-engine/src/lib.rs` (spawn do processo Python, stdio JSONL, timeout, shutdown e testes de integração)

## 3. Comandos Tauri em `mam-interface` e `src-tauri`

- [ ] 3.1 Expor `ai_health`, `ai_processar_pagina`, `ai_listar_modelos` em `backend/interface/src/lib.rs`
- [ ] 3.2 Registrar `AiEngineManager` e comandos Tauri em `backend/src-tauri/src/lib.rs`

## 4. Validação

- [ ] 4.1 Executar `cargo test --workspace`, testes Python em `ai-engine/` e `openspec validate implement-ai-engine`
