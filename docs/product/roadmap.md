# Roadmap — Manga Archive Manager (MAM)

Este documento registra a sequência de fases de desenvolvimento do MAM, alinhada à metodologia **OpenSpec (Spec-Driven Development)** e aos módulos definidos em [modules.md](../architecture/modules.md).

## Fases Concluídas

1. **Fase 1 e 2 — Fundação Técnica e Workspace** (`establish-mam-technical-foundation`, commit `dc4a469`)
   - Estrutura OpenSpec (`spec-driven`, `pt-BR`), skills globais do OpenCode e documentação de produto/arquitetura.
   - Workspace Rust (`backend/Cargo.toml`), shell Tauri 2 e frontend inicial React + TypeScript.
2. **Fase 3 — Domínio e Persistência SQLite** (`2026-11-18-implement-sqlite-database`, commit `58cdbee`)
   - Crate `mam-database` com schema, migrações versionadas, modelos e operações CRUD com caminhos relativos.
3. **Fase 4 — Gerenciamento de Capítulos e Storage** (`2026-11-18-implement-capitulo-management`, commit `37b571d`)
   - Crates `mam-chapter` e `mam-storage` com detecção de `.cbz`, parsing de páginas, integridade SHA-256 e comandos Tauri.
4. **Fase 5 — Archive Manager (CBZ)** (`2026-11-18-implement-archive-cbz`, commit `fca241a`)
   - Crate `mam-archive` com criação, leitura, validação de CBZ e escrita atômica (`temp` + `rename`).
5. **Fase 6 — Integração com Fontes Autorizadas** (`2026-11-18-implement-source-integration`, commits `52b99b6`, `264637a`, `3197417`)
   - Crate `mam-source-integration` com trait `Fonte`, busca, listagem de capítulos, detecção de duplicatas e operações com progresso.

## Próximas Fases

6. **Fase 7 — Library Management (`mam-library`)** *(próximo passo)*
   - Catálogo de obras, volumes e capítulos, metadados, organização híbrida de volumes e gerenciamento de raízes de biblioteca.
7. **Fase 8 — CBZ Reader & Interface Desktop (`backend/frontend`)**
   - Leitor integrado (navegação, zoom, modo dupla página, favoritos, marcadores, histórico) e integração completa com comandos/eventos Tauri.
8. **Fase 9 — Transfer Manager & Synchronization (`mam-transfer`)**
   - Cópia integral, transferência seletiva, comparação e sincronização entre bibliotecas sem exclusão automática da origem.
9. **Fase 10 — Manga AI Engine (`mam-ai-engine` + processo Python)**
   - Comunicação JSON-RPC 2.0 sobre stdio, OCR, detecção de balões, tradução local (ONNX Runtime/PyTorch), revisão e gerenciamento opcional de modelos.
10. **Fase 11 — Empacotamento Portátil (`packaging/`)**
    - Geração de pacotes portáteis por SO e arquitetura: ZIP (Windows) e AppImage / TAR.GZ (Linux).
