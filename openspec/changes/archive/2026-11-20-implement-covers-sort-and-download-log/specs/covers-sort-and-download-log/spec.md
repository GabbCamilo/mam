# Specification — covers-sort-and-download-log

## ADDED Requirements

### Requirement: Proxy e salvamento local de capas das obras
The system SHALL fetch cover images via the Rust backend (`source_obter_capa`) returning a Base64 Data URL and persist `cover.jpg` locally in the work directory when archiving chapters.

#### Scenario: Codificar capa e listar resumo com capa local
- **WHEN** uma obra possui `cover.jpg` em sua pasta relativa na biblioteca
- **THEN** `library_listar_obras_resumo` SHALL retornar `capa_data_url` preenchida e `total_capitulos` correspondente ao banco local.

### Requirement: Registro e listagem do histórico de downloads
The system SHALL record every archived chapter in `historico_operacoes` with `tipo = "download"` and expose `source_listar_historico_downloads` with name, destination folder, and timestamp.

#### Scenario: Consultar histórico após arquivamento
- **WHEN** um capítulo é arquivado via fonte ou extrator modular
- **THEN** `source_listar_historico_downloads` SHALL listar o registro contendo nome, pasta de destino e data/hora.
