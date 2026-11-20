# ux-improvements Specification

## Purpose
TBD - created by archiving change implement-ux-improvements-and-longstrip. Update Purpose after archive.

## Requirements

### Requirement: Exclusão de capítulos arquivados e status Lido/Não lido
The system SHALL allow deleting archived chapters from the library catalog and toggling `Lido` / `Não lido` status per chapter.

#### Scenario: Excluir capítulo arquivado
- **WHEN** `library_excluir_capitulo` é chamado para um capítulo existente
- **THEN** o registro MUST ser removido da tabela `capitulos` e, caso não haja outro registro referenciando o mesmo arquivo, o `.cbz` correspondente SHALL ser removido se solicitado.

### Requirement: Idioma e miniatura de capa na busca MangaDex e modo Long strip contínuo
The system SHALL expose chapter language codes and cover thumbnail URLs from MangaDex search results, and render all chapter pages vertically in `Long strip` mode inside `CbzReader`.

#### Scenario: Alternar para Long strip no leitor
- **WHEN** o usuário seleciona `Long strip` no modal de Layout do `CbzReader`
- **THEN** o leitor SHALL carregar e exibir todas as páginas do capítulo em sequência vertical contínua para rolagem infinita.
