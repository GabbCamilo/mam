# Módulos — Manga Archive Manager

## Core

Responsabilidades: orquestração, ciclo de vida da aplicação, configuração portátil, inicialização de processos auxiliares.

## Library Manager

Responsabilidades: catálogo de obras, volumes e capítulos; metadados; organização híbrida de volumes; caminhos relativos à raiz da biblioteca.

## Chapter Manager

Responsabilidades: detecção de capítulos, identificação de páginas, integridade por capítulo, detecção de páginas ausentes.

## Storage Manager

Responsabilidades: identificação de bibliotecas MAM, reconhecimento de unidades conectadas, verificação de espaço, gerenciamento de arquivos existentes, detecção de duplicatas, validação de integridade (SHA-256), registro de operações, recuperação de operações interrompidas.

## Source Integration

Responsabilidades: identificação de fontes disponíveis, pesquisa de obras, identificação de capítulos, seleção individual/múltipla, gerenciamento de operações, detecção de conteúdos já existentes, verificação de páginas ausentes, obtenção autorizada (sem DRM).

## Archive Manager

Responsabilidades: criação de arquivos CBZ, validação de arquivos CBZ, identificação de páginas, consulta de quantidade de páginas, preservação de imagens originais, escrita segura com arquivos temporários.

## CBZ Reader

Responsabilidades: abertura de CBZ, leitura de páginas sob demanda, navegação entre páginas e capítulos, leitura vertical e horizontal, zoom, ajuste à tela, modo página dupla, histórico da última página, favoritos, marcadores, navegação entre capítulos consecutivos.

## Manga AI Engine

Responsabilidades: OCR, detecção de balões, tradução (sobreposição e substituição visual), revisão, gerenciamento de modelos, processamento local exclusivamente.

## Transfer Manager

Responsabilidades: cópia integral de biblioteca, transferência seletiva (mangás, volumes, capítulos), validação de integridade, relatórios de operação.

## Synchronization

Responsabilidades: comparação de duas bibliotecas, identificação de arquivos ausentes ou diferentes, atualização de destino, evitação de sobrescritas inesperadas, preservação de metadados e estrutura.

## Interface

Responsabilidades: interface da biblioteca, navegação, leitor, painéis administrativos, visualização de progresso, gerenciamento de modelos, apresentação de resultados de IA.

## Regras de dependência

- Frontend acessa apenas via comandos e eventos Tauri (não acessa processo Python diretamente)
- Backend Rust controla inicialização, comunicação, encerramento e recuperação do AI Engine
- Não criar microsserviços
