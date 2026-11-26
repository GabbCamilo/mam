# Delta Spec: Assisted Learning, Auto-Capture & Single Chapter Target

## ADDED Requirements

### Requirement: Memória Progressiva Cross-Site
O sistema SHALL manter uma memória persistente e portátil em `<MAM_LIBRARY>/System/connectors/ai_memory.json` com padrões estruturais aprendidos (templates de URL, seletores de links de capítulos e seletores de imagens) ranqueados por acertos e falhas.

#### Scenario: Reutilização de padrão aprendido em site inédito
- **WHEN** o usuário tenta descobrir capítulos ou extrair imagens de um domínio sem conector próprio em `sites.json`
- **THEN** o extrator consulta os padrões de maior confiança em `ai_memory.json` antes do fallback genérico bruto e reforça o score em caso de sucesso.

### Requirement: Aprendizado Profundo DOM + URL na Janela Assistida
O sistema SHALL aprender padrões de URL de capítulos, seletores de links na página da obra e seletores de imagens na página do capítulo a partir da navegação na Janela Assistida.

#### Scenario: Aprendizado estrutural completo ao navegar de obra para capítulo
- **WHEN** a Janela Assistida registra a página da obra e a página de um capítulo do mesmo domínio
- **THEN** o sistema infere o padrão de URL (incluindo segmentos numéricos e query params), identifica o seletor de links no DOM da obra e registra os seletores de imagem do capítulo em `sites.json` e `ai_memory.json`.

### Requirement: Auto-Captura Segura na Janela Assistida
O sistema SHALL observar o término de carregamento na Janela Assistida e capturar automaticamente URL, HTML e imagens quando a página estiver pronta e fora de telas de verificação humana, emitindo evento para a UI.

#### Scenario: Auto-captura e auto-importação após carregamento válido
- **WHEN** o usuário ativa a auto-captura/auto-importação e uma página válida termina de carregar na Janela Assistida
- **THEN** o backend emite `assisted-window://captured` para o frontend, preenchendo ou disparando automaticamente a descoberta/importação sem exigir clique manual em Capturar.

#### Scenario: Página em verificação humana não é capturada prematuramente
- **WHEN** a Janela Assistida exibe tela de desafio humano (ex.: Cloudflare)
- **THEN** o sistema aguarda passivamente sem tentar contornar o desafio e não dispara auto-captura até a página real carregar.

### Requirement: Seleção de Destino no Modo Capítulo Único
O sistema SHALL permitir que o usuário escolha no modo Capítulo Único / URL Direta entre detecção automática, um mangá existente na biblioteca ou a criação de uma nova obra/coleção para capítulos unitários.

#### Scenario: Adicionar capítulo único a um mangá existente
- **WHEN** o usuário seleciona uma obra existente no seletor de destino e importa uma URL de capítulo único
- **THEN** o `.cbz` é salvo na pasta dessa obra e o capítulo é vinculado ao `ObraId` selecionado no SQLite.

#### Scenario: Criar nova obra/coleção para capítulos unitários
- **WHEN** o usuário escolhe criar uma nova obra ou usa o atalho de coleção de unitários
- **THEN** a nova obra é criada/reutilizada na biblioteca e recebe o capítulo importado.
