# Manga Archive Manager (MAM)

Aplicação desktop multiplataforma (Windows e Linux, x86-64) para gerenciamento, arquivamento, organização, leitura, transferência e processamento inteligente de mangás e quadrinhos digitais.

## Objetivo

Proporcionar uma biblioteca digital independente do computador, armazenada em SSD externo, acessível em diferentes sistemas operacionais sem depender de servidores ou bancos de dados remotos.

## Stack

- **Tauri 2** — aplicação desktop
- **React + TypeScript** — frontend (`backend/frontend/`)
- **Rust** — backend workspace modular (`backend/`)
- **SQLite** — banco de dados local (`System/database.db` com caminhos relativos)
- **Python 3.11+** — Manga AI Engine (`ai-engine/` via JSON-RPC 2.0 sobre stdio)
- **ONNX Runtime / PyTorch** — inferência local (CPU obrigatório, GPU opcional)

## Portabilidade em SSD Externo

- Todos os caminhos gravados no SQLite são **relativos à raiz da biblioteca**.
- Você pode transferir apenas a pasta da sua biblioteca (com `System/database.db` e os arquivos `.cbz`) para outro SSD ou computador.
- Resolução automática de raiz no `src-tauri`:
  1. `MAM_LIBRARY_DIR` (variável de ambiente explícita, se definida);
  2. `System/database.db` ou `MAM_LIBRARY/` ao lado do executável portátil;
  3. Diretório de dados da aplicação (`app_data_dir`) em ambiente de desenvolvimento.
- Descoberta do AI Engine: `MAM_AI_ENGINE_DIR`, `<exe_dir>/ai-engine` ou pasta `ai-engine/` do repositório.

## Desenvolvimento e Testes

```powershell
# Frontend
cd backend/frontend
npm install
npm run build

# Backend Rust (workspace completo)
cd backend
cargo test --workspace

# AI Engine Python
cd ai-engine
python -m unittest discover -s tests -v
```

## Empacotamento Portátil

Consulte [packaging/README.md](packaging/README.md):
- **Windows (x86-64 ZIP)**: `.\packaging\windows\build-portable-zip.ps1`
- **Linux (x86-64 AppImage / TAR.GZ)**: `./packaging/linux/build-portable-linux.sh`

## Metodologia e Documentação

OpenSpec (Spec-Driven Development) com schema `spec-driven` em `openspec/`. Consulte `docs/` para visão do produto, arquitetura, roadmap e glossário.

## Licença

Privado — uso interno.
