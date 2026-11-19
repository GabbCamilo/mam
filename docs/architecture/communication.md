# Comunicação entre Módulos — Manga Archive Manager

## Mecanismos

1. **Chamadas internas** entre módulos do backend Rust.
2. **Eventos** para informar progresso e mudanças de estado.
3. **Comandos e eventos Tauri** para comunicação com o frontend.

## Comunicação Frontend ↔ Backend

- Via comandos e eventos Tauri
- Frontend não acessa diretamente o processo Python
- Backend Rust controla a inicialização, comunicação, encerramento e recuperação do AI Engine

## Comunicação Rust ↔ Python (AI Engine)

- Protocolo: JSON-RPC 2.0 sobre stdio
- Formato: mensagens JSON delimitadas por linhas (JSONL)
- Rust inicia o executável auxiliar Python e controla sua comunicação

### Operações conceituais

- `engine.health`
- `engine.shutdown`
- `engine.cancel`
- `models.list`
- `models.inspect`
- `ocr.process`
- `bubbles.detect`
- `translation.process`
- `image.clean`

### Estrutura de cada operação

- Identificador de requisição
- Parâmetros tipados
- Resultado estruturado
- Erros padronizados
- Identificador de tarefa
- Estado de processamento
- Possibilidade de cancelamento (quando aplicável)

### Regras

- Não transmitir imagens grandes em Base64
- Utilizar arquivos temporários e referências controladas aos arquivos de entrada e saída
- Processo Python usa diretórios de trabalho próprios
- Não alterar arquivos originais
