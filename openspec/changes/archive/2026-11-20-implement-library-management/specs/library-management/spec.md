# Specification — library-management

## ADDED Requirements

### Requirement: Gerenciamento do catálogo de obras
The system SHALL allow creating, listing, querying, and updating works (`Obra`) in the local library catalog using paths relative to the library root.

#### Scenario: Criar nova obra com caminho relativo
- **WHEN** o usuário ou módulo solicita o cadastro de uma obra com título e caminho relativo válido
- **THEN** o `LibraryManager` MUST validar que o caminho é relativo (sem letra de unidade ou prefixo absoluto), persistir a obra no SQLite via `mam-database` e retornar o registro criado.

#### Scenario: Rejeitar caminho absoluto ao cadastrar obra
- **WHEN** uma tentativa de cadastro fornece um caminho absoluto (ex.: `C:\...` ou `/mnt/...`)
- **THEN** o `LibraryManager` MUST rejeitar a operação com erro de validação sem persistir no banco.

### Requirement: Organização híbrida de volumes e capítulos
The system SHALL support hybrid organization where chapters (`Capitulo`) can be associated either with a specific `Volume` or directly with an `Obra` without a volume.

#### Scenario: Listar estrutura de uma obra com e sem volumes
- **WHEN** os detalhes de uma obra são consultados
- **THEN** o `LibraryManager` SHALL retornar os volumes cadastrados (com seus respectivos capítulos ordenados) e os capítulos avulsos associados diretamente à obra.

### Requirement: Gerenciamento de metadados complementares
The system SHALL allow updating and validating complementary metadata (`metadados_complementares`) of an `Obra` in JSON format without altering original `.cbz` files.

#### Scenario: Atualizar metadados válidos de uma obra
- **WHEN** metadados JSON válidos são enviados para atualização de uma obra existente
- **THEN** o `LibraryManager` MUST validar o formato JSON, atualizar apenas o registro no banco SQLite e preservar intactos os arquivos `.cbz` da biblioteca.
