# ai-part1-onnx-and-models Specification

## Purpose
TBD - created by archiving change implement-ai-part1-onnx-and-models. Update Purpose after archive.

## Requirements

### Requirement: Contrato unificado process_page com coordenadas normalizadas
The AI Engine SHALL expose a `process_page` JSON-RPC 2.0 method returning image dimensions and detected/translated regions with normalized bounding boxes (`[x, y, w, h]` in `0.0..=1.0`) and `estado_revisao = "nao_revisado"`.

#### Scenario: Processar página de imagem existente em disco
- **WHEN** `process_page` recebe o caminho de uma imagem válida em disco
- **THEN** o AI Engine SHALL ler as dimensões reais da imagem via Pillow/NumPy sem alterá-la e retornar o payload estruturado com `provedor`, `dimensoes_imagem`, `regioes` e `estado_revisao: "nao_revisado"`.

### Requirement: Adaptador ONNX Runtime com CPU obrigatório
The AI Engine SHALL encapsulate ONNX Runtime session loading with `CPUExecutionProvider` as mandatory baseline and graceful reporting when `onnxruntime` or a model file is absent.

#### Scenario: Consultar disponibilidade de sessão ONNX sem runtime instalado
- **WHEN** `OnnxSessionAdapter` é consultado em ambiente sem `onnxruntime` ou sem arquivo `.onnx`
- **THEN** o adaptador SHALL reportar indisponibilidade de forma explícita sem lançar exceção não tratada.
