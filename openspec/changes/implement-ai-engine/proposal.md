# Proposal — implement-ai-engine

## Why

O MAM prevê processamento inteligente de mangás e quadrinhos digitais exclusivamente local via **Manga AI Engine**, executado como processo auxiliar Python controlado pelo backend Rust (`mam-ai-engine`) através de JSON-RPC 2.0 sobre stdio (JSONL), sem modificar os arquivos `.cbz` originais e sem depender de APIs em nuvem.

## What Changes

- `ai-engine/`: criar o processo auxiliar Python (`ai-engine/mam_ai_engine/server.py`, `protocol.py`, `pipeline.py`, `models_manager.py`) com loop JSON-RPC 2.0 sobre `stdin`/`stdout` (JSONL), suporte a `ping`, `ocr`, `detect_bubbles`, `translate`, `models_list`, `models_inspect` e `shutdown`.
- `backend/ai-engine/src/`: implementar `AiEngineManager`, `JsonRpcRequest`/`JsonRpcResponse`, gerenciamento do processo filho Python (`tokio::process::Command`) com inicialização, timeout, encerramento seguro e recuperação, além de registro de resultados e estado de revisão (`nao_revisado` / `revisado`).
- `backend/interface/` e `backend/src-tauri/`: expor comandos Tauri `ai_health`, `ai_processar_pagina`, `ai_listar_modelos` e `ai_revisar_resultado`.

## Capabilities

### New Capabilities

- `ai-engine`: coordenação Rust ↔ Python via JSON-RPC 2.0 sobre stdio (JSONL), pipeline local (OCR, detecção de balões, tradução, revisão manual) com preservação integral dos `.cbz` originais e gerenciamento opcional de modelos.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos criados/modificados: `ai-engine/`, `backend/ai-engine/`, `backend/interface/`, `backend/src-tauri/`.
- Nenhum modelo de IA pesado é baixado ou instalado automaticamente.
- Resultados de IA são estruturados em JSON independente dos arquivos `.cbz` originais.
