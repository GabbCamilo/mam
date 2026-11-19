# Glossário — Manga Archive Manager

## CBZ
Formato de arquivo baseado em ZIP contendo imagens sequenciais de um capítulo de mangá ou quadrinho.

## Obra
Trabalho completo (mangá ou quadrinho), composto por um ou mais volumes e/ou capítulos.

## Volume
Agrupamento de capítulos de uma obra (ex.: Volume 001).

## Capítulo
Unidade individual de uma obra, armazenada como arquivo .cbz com páginas numeradas e ordenadas.

## Biblioteca MAM
Estrutura hierárquica de armazenamento de conteúdos, organizada por obra > volume > capítulo, com raiz contendo pastas Manga/, System/, Metadata/, Integrity/, Logs/.

## Caminho relativo
Caminho de arquivo expresso em relação à raiz da biblioteca, não vinculado a letra de unidade ou computador específico.

## SHA-256
Algoritmo de hash utilizado para validação de integridade de arquivos.

## AI Engine
Subsistema de inteligência artificial do MAM, executado localmente (processo Python auxiliar).

## OCR
Reconhecimento Óptico de Caracteres — extração de texto presente em imagens.

## Balão
Região de fala ou pensamento em uma página de quadrinho/mangá.

## JSON-RPC 2.0
Protocolo de chamada de procedimento remoto baseado em JSON, utilizado para comunicação entre o backend Rust e o AI Engine Python sobre stdio.

## Stdio
Entrada/saída padrão utilizado para comunicação entre processos.

## JSONL
Formato de mensagens JSON delimitadas por linhas.

## ONNX Runtime
Mecanismo de inferência de modelos de aprendizado de máquina, prioridade do MAM.

## PyTorch
Framework de aprendizado de máquina, utilizado como adaptador complementar quando necessário.

## Cache L1/L2/L3/L4/L5
Camadas de cache: L1 memória, L2 disco, L3 processamento, L4 modelos, L5 conteúdo original.

## Model Manager
Gerenciador de modelos de IA (instalar, remover, atualizar, selecionar, inspecionar).

## Transferência seletiva
Cópia de itens específicos da biblioteca (mangás, volumes, capítulos) entre unidades.

## Sincronização
Comparação de duas bibliotecas para identificar arquivos ausentes ou diferentes, permitindo atualizar o destino.

## Cópia integral
Cópia de uma biblioteca inteira para outra unidade de armazenamento.
