# Manga AI Engine — Manga Archive Manager

## Execução

- Processo auxiliar Python, independente do backend Rust
- Não exigir instalação de Python pelo usuário final (runtime empacotado)
- Processamento exclusivamente local (sem APIs de IA em nuvem)

## Comunicação com o backend

- JSON-RPC 2.0 sobre stdio (JSONL)
- Controlado pelo backend Rust (inicialização, comunicação, encerramento, recuperação)

## Módulos

### OCR
Reconhecimento de texto presente nas páginas. Identificação de regiões de texto e extração do conteúdo reconhecido.

### Detecção de balões
Identificação e localização de balões de fala e outras regiões relevantes. Permite destacar visualmente os elementos identificados na página.

### Tradução
Duas modalidades:
1. Sobreposição: apresentar texto traduzido sem remover a imagem original.
2. Substituição visual: processar região do texto original e renderizar texto traduzido.

### Image Processing
Limpeza e preparação de imagens para processamento de IA.

## Idiomas

- Idioma de origem configurável individualmente por obra
- Suporte a múltiplos idiomas de origem em diferentes obras
- Idioma de destino configurável pelo usuário
- Tradução automática + revisão manual

## Revisão

- Tradução automática identificada como não revisada
- Tradução corrigida manualmente com estado próprio
- Correção de texto reconhecido e de tradução
- Armazenamento do resultado revisado
- Identificação do estado da tradução

## Preservação dos originais

- CBZ originais nunca modificados
- Resultados armazenados separadamente (JSON associado às páginas): identificação da página, regiões detectadas, coordenadas, texto reconhecido, idioma, texto traduzido, estado da revisão, anotações de renderização
- Ativar/desativar camadas de processamento sem alterar arquivos originais

## Gerenciamento de modelos

- Modelos opcionais; usuário instala apenas os necessários
- Instalação, remoção, atualização, seleção de modelos
- Consulta de compatibilidade, tamanho, requisitos de execução, disponibilidade, integridade
- Três locais de armazenamento: SSD da biblioteca, armazenamento local do computador, diretório personalizado
- Não instalar modelos automaticamente; instalação solicitada pelo usuário com apresentação do espaço necessário

## Inferência

- ONNX Runtime como mecanismo prioritário
- PyTorch como adaptador complementar
- Execução por CPU como caminho obrigatório
- Aceleração por GPU como recurso opcional
- Arquitetura permite substituir modelos e runtimes sem modificar o núcleo

## Cache de IA

Reutilização considerando: hash da imagem original, identificador do modelo, versão, configurações, idioma de origem, idioma de destino, versão do pipeline. Alterações relevantes invalidam o resultado.
