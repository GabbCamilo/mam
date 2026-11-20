# Proposal — implement-ai-part1-onnx-and-models

## Why

Para evoluir o Manga AI Engine de forma modular e eficiente, a **Parte 1** consolida o adaptador de inferência ONNX Runtime (CPU obrigatório + GPU opcional), o pré-processamento de imagem com Pillow/NumPy e o contrato unificado `process_page` (com `bbox` normalizado para sobreposição visual na Parte 2), mantendo ausência total de downloads automáticos.

## What Changes

- `ai-engine/mam_ai_engine/onnx_runtime.py`: criar adaptador modular para `onnxruntime.InferenceSession` (priorizando `CPUExecutionProvider` obrigatório e provedores GPU opcionais) e pré-processamento de imagem com `PIL`/`numpy`.
- `ai-engine/mam_ai_engine/pipeline.py`: adicionar `processar_pagina_completa` (`process_page`) retornando `dimensoes_imagem` e lista padronizada de regiões (`bbox` normalizado `[x, y, w, h]`, `texto_original`, `texto_traduzido`, `confianca`, `estado_revisao: "nao_revisado"`).
- `ai-engine/mam_ai_engine/models_manager.py`: adicionar cálculo opcional de SHA-256 e verificação de compatibilidade por extensão.
- `ai-engine/mam_ai_engine/server.py` e `ai-engine/tests/test_server.py`: expor método JSON-RPC `process_page` e cobrir com testes unitários.

## Capabilities

### New Capabilities

- `ai-part1-onnx-and-models`: adaptador ONNX Runtime modular, pré-processamento de imagem e contrato `process_page` com coordenadas normalizadas.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos criados/modificados apenas em `ai-engine/` e `openspec/`.
- Nenhuma instalação automática de modelos ou dependência de nuvem.
