# Change: Conector Universal Gerenciado pela IA Portátil

## Why
Atualmente os conectores declarativos (`sites.json`), a memória progressiva (`ai_memory.json`), os extratores modulares e a busca na aba **Fontes** operam de forma fragmentada: o usuário precisa escolher manualmente um site específico para buscar ou depender de regras isoladas por domínio na importação por link.

## What Changes
- **Orquestrador `ConectorUniversal` (`backend/source-integration/src/universal.rs`):**
  - Unifica todos os conectores (`sites.json` / `default_sites.json`) e a memória progressiva (`ai_memory.json`) sob uma única camada de correlação de rota para links importados.
  - Identifica automaticamente o caminho correlacionado ao link (conector por domínio → padrões ranqueados da memória → fallback estrutural `ai-engine`), registrando o método/conector utilizado e reforçando a memória em caso de sucesso.
- **Busca Unificada `"universal"` (`backend/interface/src/lib.rs`):**
  - Quando `conector_id == "universal"`, consulta em paralelo os conectores pesquisáveis ativos (MangaDex + conectores com `url_busca`), agregando resultados sem falhar caso um site individual esteja indisponível ou protegido.
- **Sincronização de Hints `ai_memory` ↔ `ai-engine` (`ai-engine/mam_ai_engine/web_reader.py`):**
  - Aceita `memory_hints` opcionais (padrões de URL e seletores ranqueados) nas chamadas de descoberta e extração de páginas.
- **UI (`backend/frontend/src/App.tsx`):**
  - Adiciona `Universal (IA Portátil)` como opção padrão no seletor de busca da aba **Fontes**, exibindo a origem de cada resultado e a rota correlacionada na importação por link.

## Out of Scope
- Contorno automatizado de CAPTCHAs ou desafios Cloudflare.
- Sincronização em nuvem ou servidores externos.
