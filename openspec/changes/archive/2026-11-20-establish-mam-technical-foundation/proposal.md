# Proposal — establish-mam-technical-foundation

## Why

O MAM necessita de uma fundação técnica consolidada antes de qualquer implementação de funcionalidade. A stack, arquitetura, plataformas, hardware mínimo, banco de dados, comunicação entre módulos e empacotamento precisam ser formalizados por meio do OpenSpec para garantir rastreabilidade, coerência e conformidade com os requisitos do produto.

## What Changes

- `openspec/config.yaml`: contexto do projeto em pt-BR, schema spec-driven
- `AGENTS.md`: instruções permanentes para sessões do agente
- `README.md`: visão geral do projeto, stack, plataformas, metodologia
- `docs/product/vision.md`, `project.md`, `scope.md`: visão, escopo e identidade do produto
- `docs/architecture/system-overview.md`, `modules.md`, `communication.md`, `storage.md`, `ai-engine.md`, `packaging.md`: arquitetura geral, domínios, comunicação, armazenamento, AI Engine, empacotamento
- `docs/glossary.md`: terminologia do projeto
- `openspec/changes/establish-mam-technical-foundation/`: Change de fundação com specs, proposal, design e tasks

## Capabilities

### New Capabilities

- `technical-foundation`: fundação técnica documentada (plataformas, hardware mínimo, stack, arquitetura, comunicação, BD, armazenamento, cache, empacotamento, testes)
- `openspec-sdd`: metodologia Spec-Driven Development configurada e funcional no repositório
- `global-skills`: seis skills globais do OpenCode para orientar agentes futuras (mam-project-context, mam-openspec-sdd, mam-architecture, mam-rust-tauri, mam-ai-engine, mam-quality-performance)

### Modified Capabilities

- (vazio — primeira especificação da fundação)

## Impact

- Arquivos criados: documentação do produto (AGENTS.md, README.md, docs/), configuração OpenSpec (openspec/config.yaml), Change de fundação (openspec/changes/establish-mam-technical-foundation/), skills globais (~/.config/opencode/skills/)
- Arquivos modificados: nenhum (repositório novo)
- Sem impacto em funcionalidades finais (nenhuma implementação de funcionalidade ocorre neste Change)
- Nenhuma dependência nova instalada (sem npm install, cargo build, pip install)

## Regras respeitadas

- Não criar ADRs nesta fase
- Não implementar funcionalidades prematuramente
- Não sobrescrever arquivos existentes sem inspecioná-los
- Toda documentação em português brasileiro
- Decisões técnicas tratadas como diretrizes aprovadas para a arquitetura inicial
