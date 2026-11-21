# Design: Leitor Web Inteligente (CDP) + Guia IA + Pipeline CBZ

## Context
O MAM já possui o `AiEngineManager` (`backend/ai-engine/src/lib.rs`) comunicando via JSON-RPC 2.0 sobre stdio com `ai-engine/mam_ai_engine/server.py`, utilizando runtime Python embarcado no pacote portátil.

## Decisions
1. **CDP em stdlib Python pura (`web_reader.py`)**: usa `socket`, `hashlib`, `base64`, `struct` e `urllib.request` para falar WebSocket CDP diretamente com `msedge.exe`/`chrome.exe` sem exigir dependências Python externas no pacote portátil.
2. **Idioma padrão `pt-br` com fallback**: tenta selecionar `Portuguese (Br)` / `pt-br` no seletor de idioma da página; suporta explicitamente `en` e `ja`, além de `all`. Caso o idioma preferido não exista, mantém o disponível.
3. **Paginação integral sem limite**: percorre todos os botões/links de paginação (`.npager__num`, `.pagination`, `rel="next"`, *"Load More"*) até o fim real, encerrando por detecção de ciclo (2 passos sem novas URLs).
4. **Extração completa de páginas de capítulo**: filtra `.js`/`.css`/`.svg`/logos/avatares em `eh_url_imagem_valida` e, no CDP, coleta imagens do container de leitura realizando scroll progressivo e avanço de páginas (`ArrowRight`/Next) quando o leitor estiver em modo página única.
