# Design — implement-ai-engine

## Context

`docs/architecture/ai-engine.md` define que o Manga AI Engine roda como processo auxiliar Python controlado pelo backend Rust via JSON-RPC 2.0 sobre stdio (JSONL), com ONNX Runtime prioritário, PyTorch complementar, CPU obrigatório e GPU opcional.

## Goals / Non-Goals

**Goals:**
- Definir contrato JSON-RPC 2.0 sobre stdio (JSONL) entre `mam-ai-engine` (Rust) e `ai-engine/` (Python).
- Garantir inicialização, envio de requisições, leitura de respostas, encerramento limpo (`shutdown`) e recuperação em caso de falha.
- Suportar estado de revisão (`nao_revisado` / `revisado`) e preservação dos `.cbz` originais.
- Manter execução funcional em CPU mesmo sem modelos pesados instalados (modo stub/fallback estruturado para testes).

**Non-Goals:**
- Baixar modelos volumosos automaticamente.
- Empacotar o runtime Python em AppImage/ZIP nesta fase (pertence à Fase 11 — Empacotamento).

## Decisions

1. **Protocolo JSONL sobre stdio**: Cada mensagem é uma linha JSON terminada por `\n`, evitando problemas de framing binário.
2. **Arquitetura modular em `ai-engine/mam_ai_engine/`**: Separação entre `protocol.py` (JSON-RPC 2.0), `pipeline.py` (OCR, balões, tradução com adaptador ONNX/CPU), `models_manager.py` (inspeção local sem download automático) e `server.py` (entrypoint stdio).
3. **Sem imagens em Base64 no JSON-RPC quando houver caminho local**: Passar caminho de imagem/página ou metadados estruturados conforme `docs/architecture/communication.md`.

## Risks / Trade-offs

- [Ambiente Python do desenvolvedor] Em testes sem `onnxruntime` instalado, o adaptador CPU do pipeline retorna estrutura válida indicando runtime disponível, permitindo testar o contrato JSON-RPC de ponta a ponta.
