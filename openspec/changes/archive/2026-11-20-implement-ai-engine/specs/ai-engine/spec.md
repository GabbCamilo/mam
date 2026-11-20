# Specification — ai-engine

## ADDED Requirements

### Requirement: Comunicação JSON-RPC 2.0 sobre stdio entre Rust e Python
The system SHALL manage a local Python auxiliary process (`ai-engine`) from the Rust backend using JSON-RPC 2.0 over stdio (one JSON object per line / JSONL), supporting lifecycle control (`start`, `ping`, `shutdown`).

#### Scenario: Inicializar processo auxiliar e responder ping
- **WHEN** o backend Rust inicia o `AiEngineManager` e envia uma requisição JSON-RPC `ping`
- **THEN** o processo Python SHALL responder via `stdout` com resultado válido contendo status e provedor de execução (`cpu` obrigatório) dentro do timeout configurado.

### Requirement: Processamento local sem alterar CBZ originais
The system SHALL execute OCR, bubble detection, and translation locally via structured JSON-RPC requests without modifying original `.cbz` archives and without calling cloud AI APIs.

#### Scenario: Processar página e retornar resultado com estado nao_revisado
- **WHEN** uma solicitação de OCR/detecção de balões/tradução é enviada para uma página
- **THEN** o sistema SHALL retornar regiões, coordenadas, texto reconhecido, texto traduzido e marcar o estado da revisão como `nao_revisado`, preservando o arquivo `.cbz` original intacto.

### Requirement: Gerenciamento opcional de modelos sem instalação automática
The system SHALL inspect and list optional AI models across configured storage locations without automatically downloading or installing models.

#### Scenario: Consultar modelos disponíveis sem instalação automática
- **WHEN** o usuário solicita a listagem ou inspeção de modelos
- **THEN** o sistema SHALL retornar o catálogo/estado dos modelos encontrados localmente sem iniciar downloads automáticos.
