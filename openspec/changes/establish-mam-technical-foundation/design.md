# Design — establish-mam-technical-foundation

## Context

Este Change estabelece a fundação técnica do Manga Archive Manager (MAM) com base nos requisitos aprovados: aplicação desktop multiplataforma (Windows e Linux), monólito modular em Rust + Tauri 2, frontend React + TypeScript, banco SQLite local, AI Engine em Python como processo auxiliar via JSON-RPC 2.0 sobre stdio, inferência com ONNX Runtime (prioritário) e PyTorch (complementar), empacotamento portátil (ZIP Windows, AppImage/TAR.GZ Linux), e hardware mínimo de referência (i7-6700K, 16 GB RAM, RX 7600, 8 GB VRAM, x86-64).

## Goals / Non-Goals

**Goals:**
- Formalizar plataformas suportadas (Windows, Linux; macOS fora de escopo)
- Formalizar hardware mínimo de referência
- Formalizar stack tecnológica aprovada
- Formalizar arquitetura geral (monólito modular + processo auxiliar Python)
- Formalizar comunicação entre módulos (internas, eventos, Tauri; JSON-RPC 2.0 stdio Rust↔Python)
- Formalizar banco de dados SQLite e regras de persistência
- Formalizar armazenamento, caminhos relativos, integridade SHA-256
- Formalizar cache multicamadas (L1-L5)
- Formalizar empacotamento portátil por SO e arquitetura
- Formalizar estratégia de testes (desempenho, integridade, recuperação, multiplataforma)
- Configurar OpenSpec (schema spec-driven) e skills globais do OpenCode

**Non-Goals:**
- Implementar qualquer funcionalidade final (biblioteca, leitor, IA, transferência, etc.)
- Criar ADRs
- Instalar dependências (npm, cargo, pip) neste Change
- Definir versões exatas de Rust, Node.js, Python, Tauri (questões técnicas pendentes)

## Decisions

1. **Plataformas: Windows e Linux apenas** — macOS fora de escopo. Pacotes separados por SO e arquitetura x86-64.
2. **Stack: Tauri 2 + React + TypeScript + Rust + SQLite + Python (AI Engine)** — stack aprovada como diretriz de arquitetura inicial.
3. **Banco: SQLite** — arquivo único, sem servidor externo, adequado à portabilidade. WAL avaliado com atenção à portabilidade e transferência.
4. **Comunicação Rust↔Python: JSON-RPC 2.0 sobre stdio (JSONL)** — permite controle total do ciclo de vida do AI Engine pelo backend Rust sem expor imagens grandes em Base64.
5. **Inferência: ONNX Runtime prioritário, PyTorch complementar, CPU obrigatório, GPU opcional** — arquitetura não amarrada a um único framework.
6. **Arquitetura: monólito modular** — domínios com responsabilidades explícitas, sem microsserviços.
7. **Caminhos relativos à raiz da biblioteca** — portabilidade entre letras de unidade (D: → E:).
8. **Cache multicamadas L1-L5** — L1 memória LRU, L2 disco, L3 processamento, L4 modelos persistentes, L5 original nunca descartável.
9. **Empacotamento: ZIP (Windows), AppImage/TAR.GZ (Linux)** — pacotes portáteis, usuário não instala Python/Node/SQLite.
10. **IA local exclusivamente** — sem APIs em nuvem; CBZ originais nunca modificados.

## Arquitetura

```
MAM
├── Desktop Application (Tauri 2, React + TypeScript)
├── Backend Core (Rust workspace, monólito modular)
├── Database (SQLite)
├── Storage Engine (Filesystem, CBZ, Hashing SHA-256, Cache L1-L5)
└── Manga AI Engine (Python processo auxiliar, JSON-RPC 2.0 stdio)
```

## Domínios e responsabilidades

1. Core — orquestração, ciclo de vida, configuração portátil
2. Library Manager — catálogo obra/volume/capítulo, metadados, organização híbrida de volumes
3. Chapter Manager — detecção de capítulos, páginas, integridade por capítulo
4. Storage Manager — bibliotecas, unidades, espaço, duplicatas, SHA-256, recuperação
5. Source Integration — busca modular, obtenção autorizada (sem DRM)
6. Archive Manager — geração CBZ
7. CBZ Reader — leitura desacoplada, navegação, zoom, favoritos, histórico
8. Manga AI Engine — OCR, balões, tradução, revisão, Model Manager
9. Transfer Manager — cópia integral/seletiva
10. Synchronization — comparação de bibliotecas, relatórios
11. Interface — shell desktop via Tauri

## Regras obrigatórias formalizadas

- Nunca excluir automaticamente arquivos da origem
- Caminhos relativos à raiz da biblioteca
- Preservar CBZ originais durante processamento de IA
- Processamento de IA exclusivamente local
- Não instalar modelos automaticamente
- Não criar ADRs nesta fase

## Risks / Trade-offs

- [Versões não definidas] Versões exatas de Rust, Node.js, Python, Tauri ainda não definidas. Mitigação: registrar como questão técnica pendente; definir antes do Apply da fase de infraestrutura.
- [Biblioteca CBZ] Biblioteca de leitura/criação CBZ em Rust ainda não selecionada. Mitigação: avaliar na fase de armazenamento.
- [Provedores de inferência GPU] Suporte da RX 7600 pode diferir entre Windows e Linux. Mitigação: verificar por runtime e provedor; CPU como caminho obrigatório.
- [Empacotamento Python] Formato do runtime Python empacotado ainda não definido. Mitigação: avaliar na fase de empacotamento.

## Migration Plan

1. Validar o Change `establish-mam-technical-foundation` com `openspec validate`
2. Aprovar a fundação técnica com as partes interessadas
3. Iniciar a Fase 1 de implementação (fundação e infraestrutura) apenas após aprovação
4. Arquivar o Change após validação

## Open Questions

1. Versões compatíveis de Rust, Node.js, Python e Tauri
2. Organização exata dos crates do workspace Rust
3. Biblioteca de acesso SQLite e estratégia de migrações
4. Biblioteca de leitura e criação CBZ
5. Implementação do cache LRU (L1 memória)
6. Gerenciamento de tarefas concorrentes (filas, cancelamento)
7. Contrato completo JSON-RPC 2.0 (esquemas de cada método)
8. Empacotamento dos executáveis Python
9. Provedores de inferência para RX 7600 em Windows e Linux
10. Modelos compatíveis com os idiomas escolhidos
11. Gerenciamento de atualizações
12. Requisitos de assinatura e distribuição
13. Estratégia de testes de compatibilidade
14. Política de retenção de logs e arquivos temporários
