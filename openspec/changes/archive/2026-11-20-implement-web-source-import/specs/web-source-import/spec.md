# Specification — web-source-import

## ADDED Requirements

### Requirement: Adaptador de fonte pública autorizada e registro no Tauri
The system SHALL register at least one authorized public source adapter (`eh_autorizada() == true`) in `SourceIntegrationManager` without any DRM, paywall, or authentication bypass.

#### Scenario: Listar fontes autorizadas disponíveis
- **WHEN** o frontend ou backend consulta `source_listar_fontes`
- **THEN** o sistema SHALL retornar as fontes autorizadas registradas com `autorizada: true`.

### Requirement: Arquivamento real de capítulos em CBZ na biblioteca
The system SHALL download chapter content from an authorized source, persist it atomically as a `.cbz` file using relative paths under the library root, and register the `Obra`, `Capitulo`, and SHA-256 integrity in SQLite.

#### Scenario: Arquivar capítulo de fonte autorizada em diretório temporário
- **WHEN** `arquivar_para_biblioteca` executa para uma obra e capítulo de uma fonte autorizada
- **THEN** o arquivo `.cbz` SHALL existir no caminho relativo esperado, o hash SHA-256 SHALL estar registrado em `integridade_arquivo` e o progresso da operação SHALL atingir 100%.

### Requirement: Painel Fontes & Importação no frontend
The frontend SHALL provide a **Fontes & Importação** tab allowing users to list sources, search works, view chapters, check duplicates, and start archiving operations.

#### Scenario: Navegar para Fontes & Importação
- **WHEN** o usuário seleciona a aba `Fontes & Importação`
- **THEN** a interface SHALL exibir as fontes registradas, campo de busca de obras, lista de capítulos com seleção e botão de arquivamento.
