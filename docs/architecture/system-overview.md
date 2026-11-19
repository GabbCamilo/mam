# Visão Arquitetural do Sistema — Manga Archive Manager

## Estilo arquitetural

Monólito modular com processo auxiliar especializado para IA (Python).

Não transformar cada domínio em serviço independente. Não criar microsserviços.

## Estrutura conceitual

```
MAM
│
├── Desktop Application
│   ├── React
│   ├── TypeScript
│   └── Tauri
│
├── Backend Core (Rust)
│   ├── Domain
│   ├── Application
│   ├── Infrastructure
│   └── Modules
│
├── Database (SQLite)
│
├── Storage Engine
│   ├── Filesystem
│   ├── CBZ
│   ├── Hashing
│   └── Cache
│
└── Manga AI Engine (Python, processo auxiliar)
    ├── OCR
    ├── Bubble Detection
    ├── Translation
    ├── Image Processing
    └── Model Runtime
```

## Domínios conceituais

1. **Core** — orquestração, ciclo de vida, configuração portátil
2. **Library Manager** — catálogo de obras/volumes/capítulos, metadados, organização híbrida de volumes
3. **Chapter Manager** — detecção de capítulos, páginas, integridade por capítulo
4. **Storage Manager** — bibliotecas, unidades, espaço, duplicatas, validação SHA-256, recuperação
5. **Source Integration** — busca modular, obtenção autorizada, geração CBZ
6. **Archive Manager** — conversão para CBZ
7. **CBZ Reader** — leitura desacoplada, navegação, zoom, favoritos, histórico
8. **Manga AI Engine** — OCR, balões, tradução local, revisão, gerenciamento de modelos
9. **Transfer Manager** — cópia integral/seletiva
10. **Synchronization** — comparação de bibliotecas, relatórios
11. **Interface** — shell desktop (React + TypeScript via Tauri)

## Princípios

- Baixo acoplamento, alta coesão, responsabilidades bem definidas, interfaces explícitas
- Testabilidade, portabilidade, manutenibilidade, evolução modular
- Separação de componentes essenciais dos opcionais (modelos de IA)
