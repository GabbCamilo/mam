# Tasks — implement-ai-part1-onnx-and-models

## 1. Adaptador ONNX, Pré-processamento e Pipeline

- [x] 1.1 Criar `ai-engine/mam_ai_engine/onnx_runtime.py` com `OnnxSessionAdapter` e `carregar_imagem_rgb`
- [x] 1.2 Atualizar `ai-engine/mam_ai_engine/pipeline.py` com `processar_pagina_completa` (`process_page`)
- [x] 1.3 Atualizar `ai-engine/mam_ai_engine/models_manager.py` com verificação de SHA-256 e compatibilidade
- [x] 1.4 Expor `process_page` em `ai-engine/mam_ai_engine/server.py` e adicionar testes em `ai-engine/tests/test_server.py`

## 2. Validação

- [x] 2.1 Executar `python -m unittest discover -s tests -v`, `cargo test -p mam-ai-engine` e `openspec validate implement-ai-part1-onnx-and-models`
