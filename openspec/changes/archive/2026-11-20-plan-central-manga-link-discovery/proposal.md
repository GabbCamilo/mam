# Proposta: Descoberta Estrutural de Capítulos por Página Central e Reformulação do Modal de Importação

## Why
Obras hospedadas em sites sem API pública (como `mangafire.to/title/jw2-medaka-boxx` e `mangalivre.to/manga/chainsaw-man-pt-br/`) centralizam centenas ou milhares de capítulos em uma página principal. O modal atual mistura importação de capítulo único com descoberta de obra inteira, e a extração atual depende apenas de `<a href>` simples no HTML inicial, falhando quando os capítulos estão em blocos estruturais repetitivos, JSON embutido ou endpoints públicos auxiliares.

## What Changes
1. **Reformulação do Modal de Importação (`App.tsx`)**:
   - Separar em duas abas claras: **Página Central da Obra (Lote)** e **Capítulo Único / Lista Manual**.
2. **Motor de Análise Estrutural de Links (`source-integration/src/extractors/`)**:
   - Adaptadores estruturais por padrão de site (`mangafire.to`, `mangalivre.to` / Madara) + analisador genérico de agrupamentos repetitivos de links (`<li>`/`<a>`/`data-*` e JSON embutido em `<script>`), sem contorno de anti-bot, DRM ou CAPTCHA.
   - Integração opcional com `ai-engine` (JSON-RPC stdio) para classificar blocos de links desconhecidos quando a heurística não encontrar padrão claro.
3. **Fluxo Unificado com `Fontes & Importação`**:
   - Ao descobrir capítulos pela URL central, exibir obra, capa, idiomas e lista completa de capítulos na mesma interface do MangaDex (com filtro `LANG`, `Selecionar Todas` e `Arquivar Selecionados` em segundo plano).
