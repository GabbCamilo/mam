# Proposal — implement-source-integration

## Why

O MAM necessita de mecanismos modulares para pesquisar obras e arquivar capítulos em fontes compatíveis e autorizadas. A integração com fontes deve ser desacoplada do núcleo, permitindo adicionar novas integrações sem modificar desnecessariamente o core da aplicação. Esta fase estabelece a camada de busca e obtenção autorizada, que alimenta o Archive Manager (Fase 5) para gerar CBZ.

## What Changes

- `backend/source-integration/Cargo.toml`: novo crate com dependências (reqwest, serde, serde_json, url, mam-core, mam-database, mam-archive, thiserror, tokio)
- `backend/source-integration/src/lib.rs`: ponto de entrada, SourceIntegrationManager
- `backend/source-integration/src/fonte.rs`: trait Fonte e gerenciamento de fontes registradas
- `backend/source-integration/src/busca.rs`: pesquisa de obras em fontes autorizadas
- `backend/source-integration/src/capitulos.rs`: identificação de capítulos de uma obra
- `backend/source-integration/src/operacao.rs`: gerenciamento de operações de arquivamento, progresso, retomada
- `backend/source-integration/src/duplicatas.rs`: detecção de conteúdos já existentes no banco
- `backend/interface/src/lib.rs`: comandos Tauri para buscar obras, listar capítulos, iniciar arquivamento
- `backend/src-tauri/src/lib.rs`: registrar novos comandos no invoke_handler
- `backend/Cargo.toml`: adicionar `mam-source-integration` ao workspace members

## Capabilities

### New Capabilities

- `source-busca-obras`: pesquisa de obras em fontes autorizadas por título
- `source-listar-capitulos`: identificação de capítulos disponíveis para uma obra
- `source-detectar-duplicatas`: verificação se obra/capítulo já está arquivado
- `source-operacao-arquivamento`: gerenciamento de operações com progresso e retomada
- `source-gerenciar-fontes`: registro, listagem, ativação/desativação de fontes

### Modified Capabilities

- `mam-interface`: novos comandos Tauri (buscar obras, listar capítulos, iniciar arquivamento)

## Impact

- Arquivos criados: `backend/source-integration/` (Cargo.toml, src/*)
- Arquivos modificados: `backend/Cargo.toml`, `backend/interface/src/lib.rs`, `backend/src-tauri/src/lib.rs`
- Nenhuma dependência nova instalada além de crates já declaradas (reqwest, serde, serde_json, url)

## Regras respeitadas

- Somente integrações que permitam arquivamento conforme autorizações/licenças aplicáveis
- Não desenvolver mecanismos para contornar DRM, paywalls, autenticação, controles de acesso ou restrições técnicas
- Caminhos relativos à raiz da biblioteca
- Detecção de conteúdos já existentes antes de arquivar
- Acompanhamento de progresso de operações
- Possibilidade de retomar operações interrompidas
- Registro de histórico de operações
