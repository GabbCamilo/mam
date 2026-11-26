# Design: Síntese Interna de APIs de Conectores (`smart-connector-api-synthesis`)

## 1. Arquitetura em 3 Camadas (Portátil e Offline-First)

```text
[Menu de Busca Interno / Conector Universal]
        │
        ▼
[1. ExecutorApiConector (Rust / reqwest)]  ──(tem `api_busca` válida?)──► Retorna ObraBuscaResultado (<500ms)
        │ (sem `api_busca` ou HTTP erro/403)
        ▼
[2. Sintetizador de API (ai-engine CDP)]   ──(intercepta XHR/JSON de busca)──► Gera `api_busca` + salva em `sites.json`
        │ (bloqueio humano / SPA complexo)
        ▼
[3. Janela Assistida (mam-assistido)]      ──(observador passivo fetch/XHR)──► Usuário navega/busca -> IA captura endpoint e sintetiza `api_busca`
```

## 2. Schema Estendido de Conector (`backend/source-integration/src/connectors.rs`)
Adicionar campos opcionais com `#[serde(default)]` em `ConectorSiteConfig` (100% retrocompatível):
- `pub api_busca: Option<ApiBuscaConfig>`
  - `endpoint: String` (com `{query}`)
  - `metodo: String` (`"GET"` padrão)
  - `formato: String` (`"json"` | `"html"`)
  - `caminho_itens: Option<String>` (ex.: `"data.items"` ou `"result"`)
  - `campo_id_ou_url: String` (ex.: `"id"` ou `"slug"` ou `"url"`)
  - `template_url_obra: Option<String>` (ex.: `"https://nhentai.net/g/{id}/"`)
  - `campo_titulo: String` (ex.: `"english_title"` | `"title"`)
  - `campo_capa: Option<String>` (ex.: `"thumbnail"`)
  - `template_url_capa: Option<String>`
  - `headers: Option<HashMap<String, String>>`

## 3. Sintetizador de API no `ai-engine` (`ai-engine/mam_ai_engine/api_synthesizer.py`)
- Nova ação JSON-RPC `synthesize_search_api`:
  - Abre a página do site (ou `url_busca` / homepage com barra de pesquisa) via CDP com `Network.enable`.
  - Preenche e submete o input de busca (`input[type="search"], input[name*="search"], input[name*="keyword"], input[name="q"], input[name="s"]`).
  - Captura respostas XHR/Fetch JSON durante 3s e analisa os payloads procurando arrays de objetos com campos compatíveis com título (`title`, `name`, `english_title`, `manga_title`) e identificador (`id`, `slug`, `url`, `href`).
  - Infere `caminho_itens`, `campo_titulo`, `campo_id_ou_url`, `campo_capa` e retorna um `ApiBuscaConfig` pronto para ser validado e salvo pelo Rust em `sites.json`.

## 4. Observador Passivo na Janela Assistida (`backend/src-tauri/src/lib.rs`)
- Ao carregar uma página na `mam-assistido`, um script leve opcional pode registrar os últimos endpoints JSON/XHR retornados pela própria sessão do usuário (quando o usuário pesquisa no site dentro da janela) e enviar no payload de captura, permitindo sintetizar a API mesmo em sites protegidos por verificação humana.

## 5. Execução no Menu de Busca (`backend/interface/src/lib.rs`)
- `source_buscar_obras_por_conector`:
  1. Se `conector.api_busca` estiver preenchido → executa via `ExecutorApiConector` (Rust puro).
  2. Se falhar ou não existir → tenta `synthesize_search_api` via `ai-engine` (se permitido) ou `buscar_obras_via_cdp`; se sintetizar com sucesso, persiste em `System/connectors/sites.json` e retorna os resultados para a UI.
  3. No conector **Universal**, prioriza todos os conectores com `api_busca` / API nativa em paralelo (< 2s) e complementa com os web/CDP ativos.
