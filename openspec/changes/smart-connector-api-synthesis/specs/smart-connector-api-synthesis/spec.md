# Delta Spec: Síntese Interna de APIs de Conectores (`smart-connector-api-synthesis`)

## ADDED Requirements

### Requirement: Perfil Declarativo de API de Busca (`api_busca`)
O sistema SHALL suportar um bloco declarativo `api_busca` em `ConectorSiteConfig` permitindo que conectores JSON/AJAX sejam consultados diretamente pelo backend Rust sem abrir navegador headless.

#### Scenario: Busca rápida via API declarativa registrada
- **WHEN** o usuário pesquisa uma obra em um conector (ou no modo Universal) que possui `api_busca` configurada
- **THEN** o backend executa a requisição HTTP diretamente com `reqwest`, mapeia os campos declarados para `ObraBuscaResultado` e exibe os resultados no menu de busca interno.

### Requirement: Síntese Automática de API pela IA (`ai-engine` e Janela Assistida)
O sistema SHALL ser capaz de inspecionar respostas XHR/JSON durante uma busca via `ai-engine` (CDP) ou Janela Assistida para inferir e persistir automaticamente a configuração `api_busca` de conectores sem busca configurada.

#### Scenario: Conector registrado sem busca recebe API sintetizada
- **WHEN** o usuário pesquisa em um conector já registrado que ainda não possui `api_busca` funcional
- **THEN** a IA tenta sintetizar o endpoint e o mapeamento de campos a partir das respostas XHR/JSON da busca e, em caso de sucesso, salva a nova `api_busca` em `System/connectors/sites.json` para reutilização imediata e futura.

### Requirement: Preservação de Portabilidade e Segurança
O sistema SHALL armazenar todas as APIs sintetizadas e métricas de confiança exclusivamente dentro de `System/connectors/` na raiz da biblioteca e SHALL NOT tentar contornar desafios anti-bot ou CAPTCHAs.

#### Scenario: Fallback seguro quando o site exige verificação humana
- **WHEN** a síntese automática encontra bloqueio de verificação humana (ex.: HTTP 403 / desafio Cloudflare)
- **THEN** o sistema interrompe a tentativa automatizada e orienta o uso da Janela Assistida para que o usuário conclua a verificação manualmente.
