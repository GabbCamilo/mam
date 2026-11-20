# Tasks — implement-portable-packaging

## 1. Estrutura e documentação de empacotamento

- [x] 1.1 Criar `ai-engine/requirements.txt` com dependências do runtime Python
- [x] 1.2 Criar `packaging/README.md` documentando pacotes por SO/arquitetura x86-64

## 2. Scripts portáteis Windows e Linux

- [x] 2.1 Criar `packaging/windows/build-portable-zip.ps1` para gerar pacote portátil ZIP Windows x86-64
- [x] 2.2 Criar `packaging/linux/build-portable-linux.sh` e `packaging/linux/README.md` para gerar AppImage/TAR.GZ Linux x86-64

## 3. Validação

- [x] 3.1 Validar sintaxe do script PowerShell (`build-portable-zip.ps1`), executar `openspec validate implement-portable-packaging` e `cargo test --workspace`
