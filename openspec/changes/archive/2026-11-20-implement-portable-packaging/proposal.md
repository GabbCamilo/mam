# Proposal — implement-portable-packaging

## Why

O MAM é uma aplicação desktop portátil para Windows e Linux (x86-64) que deve ser executada diretamente de diferentes unidades de armazenamento (como SSD externo) sem exigir instalação convencional nem que o usuário instale Python, Node.js ou SQLite. A pasta `packaging/` ainda não existe e precisa ser estruturada com scripts e documentação de empacotamento reproduzível.

## What Changes

- `ai-engine/requirements.txt`: formalizar dependências do runtime Python empacotado.
- `packaging/README.md`: documentar layout portátil, pré-requisitos de build por SO (Windows e Linux x86-64) e execução a partir de SSD externo.
- `packaging/windows/build-portable-zip.ps1`: script PowerShell para gerar o pacote portátil `MAM-windows-x86_64.zip` contendo o binário Tauri/Rust (`app.exe`), frontend embutido, `ai-engine/` e estrutura portátil sem instalação automática de modelos.
- `packaging/linux/build-portable-linux.sh` e `packaging/linux/README.md`: script shell para gerar AppImage (prioritário) e TAR.GZ (alternativo) para Linux x86-64, documentando dependências e permissões de execução.

## Capabilities

### New Capabilities

- `portable-packaging`: empacotamento portátil por sistema operacional e arquitetura x86-64 (ZIP no Windows, AppImage e TAR.GZ no Linux) preservando portabilidade de dados e sem exigir instalação de Python/Node/SQLite pelo usuário.

### Modified Capabilities

- (nenhuma)

## Impact

- Arquivos criados: `packaging/`, `ai-engine/requirements.txt`.
- macOS permanece estritamente fora de escopo.
- Nenhum modelo volumoso de IA é empacotado ou baixado automaticamente.
