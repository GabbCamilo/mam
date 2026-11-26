# Delta Spec: Conector Universal Gerenciado pela IA Portátil

## ADDED Requirements

### Requirement: Conector Universal de Correlação por Link
O sistema SHALL unificar conectores declarativos (`sites.json`) e memória progressiva (`ai_memory.json`) em um orquestrador `ConectorUniversal` capaz de correlacionar qualquer URL importada ao conector ou padrão estrutural mais adequado.

#### Scenario: Correlação automática de link conhecido ou inédito
- **WHEN** uma URL de obra ou capítulo é enviada para descoberta ou importação
- **THEN** o `ConectorUniversal` identifica a rota correlacionada (conector por domínio, padrões da memória ou fallback `ai-engine`) e aplica os padrões combinados para capturar capítulos/páginas.

### Requirement: Busca Unificada no Conector Universal
O sistema SHALL disponibilizar o conector `Universal (IA Portátil)` na aba Fontes, agregando resultados de múltiplos conectores pesquisáveis em paralelo e tolerando falhas individuais.

#### Scenario: Busca agregada em múltiplos conectores
- **WHEN** o usuário pesquisa um título com o conector `universal` selecionado
- **THEN** o backend consulta o MangaDex e os conectores ativos com `url_busca`, retornando os resultados consolidados com identificação da fonte de origem.
