# frontend-ui-redesign Specification

## Purpose
TBD - created by archiving change redesign-frontend-dark-ui. Update Purpose after archive.

## Requirements

### Requirement: Shell desktop dark-first com alternância de tema e barra de status
The frontend SHALL render a cohesive dark-first desktop shell with a Dark/Light theme toggle, top navigation pills, quick `+ Importar Link` action, and bottom status bar while preserving all existing Tauri integrations.

#### Scenario: Alternar tema e navegar entre abas
- **WHEN** o usuário clica no botão de tema ou nas abas superiores (`Biblioteca`, `Fontes & Importação`, `Transferência & Sync`, `AI Engine`)
- **THEN** o tema visual e o painel ativo SHALL atualizar imediatamente sem recarregar a aplicação.

### Requirement: Modais flutuantes de Layout e Capítulos no CbzReader
The `CbzReader` component SHALL provide floating modals for reader layout settings (`Single`, `Double`, `Long strip`, zoom, fit mode) and quick chapter/volume switching with search and sorting.

#### Scenario: Abrir modal de Layout e modal de Capítulos no leitor
- **WHEN** o usuário aciona os botões `Layout` ou `Capítulos` na barra do leitor
- **THEN** o leitor SHALL exibir o respectivo modal flutuante mantendo a página atual preservada.
