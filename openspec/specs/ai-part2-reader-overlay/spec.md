# ai-part2-reader-overlay Specification

## Purpose
TBD - created by archiving change implement-ai-part2-reader-overlay. Update Purpose after archive.

## Requirements

### Requirement: Processamento de página CBZ e persistência em resultados_ia
The system SHALL process a specific page from a `.cbz` archive using `AiEngineManager::processar_pagina`, persist the structured JSON output in `resultados_ia` with `estado_revisao = "nao_revisado"`, clean up any temporary image file, and preserve the original `.cbz` intact.

#### Scenario: Processar página de um CBZ e salvar resultado no SQLite
- **WHEN** `ai_processar_e_salvar_pagina` é executado para uma página válida de um `.cbz`
- **THEN** o sistema SHALL retornar o `ResultadoIA` salvo com `estado_revisao = "nao_revisado"`, removendo o arquivo temporário após a inferência e mantendo o `.cbz` original inalterado.

### Requirement: Camada visual de sobreposição no CbzReader
The frontend `CbzReader` SHALL provide a toggleable visual overlay layer rendering `regioes` (`bbox: [x, y, w, h]`) over the page image without modifying the original image.

#### Scenario: Ativar sobreposição visual na página
- **WHEN** existem regiões com `bbox` e o usuário ativa a camada de sobreposição no `CbzReader`
- **THEN** o leitor SHALL posicionar as caixas de texto traduzido sobre a imagem usando coordenadas percentuais derivadas de `bbox`.
