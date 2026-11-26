# Change: Síntese Interna de APIs de Conectores pela IA Portátil (`smart-connector-api-synthesis`)

## Why
Atualmente:
1. Vários conectores já registrados em `default_sites.json` / `sites.json` (como `MangaLivre`, `Atsumaru`, etc.) ainda não possuem busca por título funcional (`url_busca: null`), e outros dependem exclusivamente de scraping HTML genérico via CDP (`search_works_cdp`), que é lento, pesado e frágil quando o site usa APIs internas/AJAX ou altera classes CSS.
2. Quando o usuário pesquisa no menu interno (ou no conector **Universal**), conectores sem `api_busca` rápida falham por timeout ou retornam listagens genéricas em vez dos resultados filtrados da obra.
3. A Janela Assistida e o `ai-engine` ainda não sintetizam automaticamente "APIs declarativas" (endpoints JSON/AJAX + mapeamento de campos + padrões de URL) a partir das requisições reais observadas durante a navegação.

## What Changes
- **Síntese Declarativa de APIs de Conectores (`ApiConectorConfig`):**
  - Estender `ConectorSiteConfig` (`sites.json`) e `MemoriaAprendizadoIA` (`ai_memory.json`) com um bloco opcional `api_busca`:
    - `endpoint`: URL template (ex.: `https://site.com/api/search?q={query}`)
    - `metodo`: `GET` | `POST`
    - `formato_resposta`: `json` | `html`
    - `caminho_lista` / `campos`: mapeamento declarativo (`id_ou_url`, `titulo`, `capa`, `template_url_obra`)
    - `headers`: `Referer` / `Accept` / `X-Requested-With`
- **Motor de Descoberta de API no `ai-engine` + Janela Assistida:**
  - Quando um conector registrado não possui `api_busca` (ou falha), o `ai-engine` (via interceptação de rede CDP) ou a Janela Assistida (via observador passivo de `fetch`/XHR durante a navegação/busca do usuário) identifica o endpoint XHR/JSON que retorna a lista de obras/capítulos e sintetiza o `ApiConectorConfig`.
- **Cliente de Busca Nativo em Rust (`ExecutorApiConector`):**
  - Na busca do menu interno (individual ou **Universal**), conectores com `api_busca` são consultados diretamente via `reqwest` em Rust (sem abrir Chrome headless), retornando `ObraBuscaResultado` em milissegundos.
  - Fallback automático para CDP (`search_works_cdp`) ou aprendizado assistido apenas quando o endpoint sintetizado não existir ou falhar.
- **Retroalimentação Portátil:**
  - Todos os perfis de API sintetizados ficam persistidos em `System/connectors/sites.json` e suas assinaturas reutilizáveis em `System/connectors/ai_memory.json`, mantendo 100% da portabilidade em SSD externo.

## Out of Scope
- Contorno automatizado de CAPTCHAs, Cloudflare Turnstile ou paywalls.
- Servidores externos ou envio de telemetria/modelos para fora da máquina local.
