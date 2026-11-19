# Manga Archive Manager (MAM)

## 1. Sobre o projeto

O Manga Archive Manager (MAM) é uma aplicação desktop multiplataforma (Windows e Linux) para gerenciamento, arquivamento, organização, leitura, transferência e processamento inteligente de mangás e quadrinhos digitais.

Seu objetivo é proporcionar uma biblioteca digital independente do computador utilizado, armazenada em SSD externo, permitindo conectar a unidade em computadores diferentes e executar o MAM sem depender de servidores ou bancos de dados remotos.

## 2. Stack tecnológica

| Camada | Tecnologia |
|--------|------------|
| Desktop | Tauri 2 |
| Frontend | React + TypeScript |
| Backend | Rust (workspace, monólito modular) |
| Banco de dados | SQLite (local, sem servidor) |
| AI Engine | Python (processo auxiliar, JSON-RPC 2.0 sobre stdio) |
| Inferência | ONNX Runtime (prioritário), PyTorch (complementar) |
| Empacotamento | ZIP (Windows), AppImage / TAR.GZ (Linux) |

## 3. Metodologia

O projeto utiliza **OpenSpec** (Spec-Driven Development) com o schema padrão `spec-driven`. Toda funcionalidade relevante passa por: Explore → Propose → Specify → Design → Tasks → Apply → Verify → Archive.

Não criar ADRs nesta fase.

## 4. Plataformas

- Windows (suportado)
- Linux (suportado)
- macOS (fora de escopo)

## 5. Hardware mínimo de referência

| Componente | Referência |
|------------|------------|
| CPU | Intel Core i7-6700K (4c/8t, 4,00 GHz) |
| RAM | 16 GB DDR4 |
| GPU | AMD Radeon RX 7600 (8 GB VRAM) |
| Arquitetura | x86-64 |
| Armazenamento | SSD |

## 6. Estrutura do repositório

```
MAM/
├── AGENTS.md
├── README.md
├── openspec/
│   ├── config.yaml
│   ├── specs/
│   └── changes/
│       └── archive/
├── docs/
│   ├── product/
│   ├── architecture/
│   ├── requirements/
│   ├── development/
│   └── glossary.md
├── apps/
├── backend/
├── ai-engine/
├── packaging/
└── tests/
```

## 7. Regras de trabalho

- Não inventar requisitos
- Não transformar sugestões em decisões aprovadas
- Não escolher tecnologias prematuramente
- Não criar complexidade arquitetural sem necessidade
- Não implementar funcionalidades não especificadas
- Não sobrescrever arquivos existentes sem inspecioná-los
- Não criar ADRs nesta fase
- Não executar comandos destrutivos sem autorização explícita
- Não fazer commits, pushes ou alterações remotas sem autorização
- Toda documentação em português brasileiro

## 8. Skills globais do OpenCode

Localizadas em `~/.config/opencode/skills/`:
- mam-project-context
- mam-openspec-sdd
- mam-architecture
- mam-rust-tauri
- mam-ai-engine
- mam-quality-performance
