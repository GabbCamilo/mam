# Change: Aprendizado Progressivo da Janela Assistida, Auto-Captura e Destino de Capítulo Único

## Why
Atualmente:
1. O aprendizado da Janela Assistida (`aprender_padrao_url`) compara apenas dois paths ou 6 query keys fixas, sem inspecionar o DOM da obra/capítulo nem acumular conhecimento reutilizável para outros sites sem conector.
2. O usuário precisa alternar manualmente entre a Janela Assistida e o MAM para clicar em **Capturar da Janela**, **Descobrir Capítulos** e **Ensinar Capítulo**.
3. No modo **Capítulo Único / URLs Manuais**, não há como escolher um mangá existente na biblioteca para receber o capítulo avulso nem criar uma nova obra/coleção dedicada diretamente no fluxo.

## What Changes
- **Memória Progressiva Cross-Site (`System/connectors/ai_memory.json`):**
  - Persistir padrões estruturais generalizados (templates de URL, seletores de links de capítulos e seletores/atributos de imagens de leitura) com contadores de acertos, falhas e score de confiança.
  - Consultar a memória em sites sem conector cadastrado e reforçar/penalizar padrões conforme o resultado de cada extração ou sessão assistida.
- **Aprendizado Profundo de Conectores (DOM + URL):**
  - Evoluir o aprendizado para aceitar `{ url_obra, html_obra }` e `{ url_capitulo, html_capitulo }`, inferindo padrões de URL numéricos/regex, `seletor_link` e `seletores_imagem` automaticamente.
  - Detectar transições obra → capítulo na Janela Assistida para aprendizado passivo.
- **Auto-Captura e Auto-Importação na Janela Assistida:**
  - Observar carregamentos e estabilização do DOM na janela `mam-assistido` (aguardando passivamente caso haja verificação humana/Cloudflare, sem nunca burlá-la).
  - Emitir eventos Tauri (`assisted-window://captured`) com `{ url, html, urls_imagens }` para o frontend React, suportando toggle de auto-captura/auto-importação.
- **Destino no Modo Capítulo Único:**
  - Permitir escolher entre: **Detectar automaticamente**, **Mangá existente na biblioteca** (por `obra_id_destino`) ou **Criar novo mangá / coleção de capítulos unitários**.

## Out of Scope
- Qualquer tentativa de contornar, resolver ou burlar CAPTCHAs ou desafios anti-bot.
- Criação de ADRs ou serviços remotos de aprendizado.
