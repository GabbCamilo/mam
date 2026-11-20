# Specification — cbz-reader

## ADDED Requirements

### Requirement: Leitura sob demanda de páginas CBZ
The system SHALL read individual page bytes from a `.cbz` archive on demand without permanently extracting images to disk and without modifying the original `.cbz` file.

#### Scenario: Ler página válida de um CBZ
- **WHEN** o leitor solicita uma página pelo índice ou nome dentro de um arquivo `.cbz` válido
- **THEN** o backend MUST extrair apenas a entrada solicitada em memória, identificar o tipo MIME da imagem e retornar os dados sem gravar arquivos temporários permanentes em disco.

#### Scenario: Solicitar índice de página inexistente
- **WHEN** o leitor solicita um índice fora dos limites das páginas de imagem do `.cbz`
- **THEN** o backend MUST retornar um erro explícito sem encerrar a aplicação.

### Requirement: Persistência de histórico, favoritos e marcadores
The system SHALL persist reading progress (last read page per chapter), favorite works/chapters, and page bookmarks in the local SQLite database.

#### Scenario: Salvar e recuperar última página lida
- **WHEN** o usuário navega para uma página de um capítulo e o progresso é salvo
- **THEN** ao reabrir o capítulo posteriormente, o sistema SHALL recuperar a última página registrada.

#### Scenario: Alternar favorito e adicionar marcador
- **WHEN** o usuário marca uma obra/capítulo como favorito ou adiciona um marcador em uma página
- **THEN** o estado SHALL ser persistido no SQLite local e refletido nas consultas seguintes.

### Requirement: Interface do leitor CBZ e navegação da biblioteca
The frontend SHALL provide a library view and an integrated CBZ reader supporting vertical, horizontal, and double-page modes, zoom, fit-to-screen, and navigation between pages and consecutive chapters.

#### Scenario: Alternar modo de leitura e zoom no frontend
- **WHEN** o usuário altera o modo de leitura (vertical, horizontal, dupla página) ou o nível de zoom/ajuste à tela
- **THEN** a visualização do leitor MUST atualizar o layout imediatamente mantendo a página atual selecionada.
