# Design — implement-ai-part1-onnx-and-models

## Context

Parte 1 de 3 da evolução do Manga AI Engine:
- **Parte 1 (este change)**: Adaptador ONNX Runtime, pré-processamento PIL/NumPy, contrato `process_page` e inspeção de modelos.
- **Parte 2 (próxima)**: Integração no `CbzReader` com camada visual de sobreposição por `bbox` e persistência em `resultados_ia`.
- **Parte 3 (futura)**: Revisão avançada por balão e cache de IA por hash.

## Decisions

1. **Coordenadas normalizadas `[x, y, w, h]` (0.0 a 1.0)**: Desacopla o posicionamento visual no frontend do zoom/resolução da tela.
2. **Importação dinâmica de `onnxruntime`**: Permite que o servidor JSON-RPC funcione tanto com `onnxruntime` instalado quanto em ambientes leves com `PIL` + `numpy`.
