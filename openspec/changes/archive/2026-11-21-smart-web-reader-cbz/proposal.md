# Proposal: Leitor Web Inteligente (CDP), Extração Completa de CBZ e Guia IA Estrutural

## Why
Na importação por link de sites externos baseados em SPA ou leitores dinâmicos (`mangafire.to`, `mangalivre.to`, etc.):
1. O `--dump-dom` simples captura apenas a primeira página da lista de capítulos (ex.: 20 itens no idioma padrão), sem acionar paginação (`[1] [2] [3]...`) nem alternar para o idioma preferido (`PT-BR`, `ENG`, `JPN`).
2. O empacotamento `.cbz` pode produzir apenas 1 página (ou capturar scripts/logos) quando o leitor monta imagens sob demanda (*single-page* / *lazy loading*) ou quando `eh_url_imagem_valida` aceita URLs de CDN não-imagem (`.js`/`.css`).

## What Changes
- **Controlador CDP Inteligente (`ai-engine/mam_ai_engine/web_reader.py`)**: navegação headless via Chrome DevTools Protocol (Edge/Chrome) com suporte a seleção de idioma (`pt-br` padrão, `en`, `ja`, `all`), paginação integral sem limite artificial e captura completa de todas as páginas em leitores dinâmicos.
- **Guia IA Estrutural (`ai-engine/mam_ai_engine/pipeline.py` & `server.py`)**: métodos JSON-RPC `smart_discover_chapters`, `smart_extract_chapter_pages` e `analyze_reader_dom` com cache de perfil por domínio (`site_profiles.json`).
- **Extração & CBZ (`backend/source-integration/src/extractors/mod.rs`, `mangafire.rs`, `generico.rs`)**: correção estrita de `eh_url_imagem_valida`, integração com o `AiEngineManager` para descoberta multi-página e extração completa de páginas de capítulos.
- **Interface & Tauri (`backend/interface/src/lib.rs`, `backend/src-tauri/src/lib.rs`, `backend/frontend/src/App.tsx`)**: parâmetro `idioma_preferido` (padrão `pt-br`) no modal de importação em lote.

## Impact
- Permite importar obras completas com centenas de capítulos paginados no idioma desejado (`PT-BR`, `ENG`, `JPN`) e gerar `.cbz` com todas as páginas reais de cada capítulo.
