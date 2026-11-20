# Design — implement-portable-packaging

## Context

`docs/architecture/packaging.md` define distribuição portátil separada por SO e arquitetura x86-64: ZIP no Windows e AppImage/TAR.GZ no Linux (macOS fora de escopo).

## Goals / Non-Goals

**Goals:**
- Criar scripts reproduzíveis em `packaging/windows/` e `packaging/linux/`.
- Garantir que os pacotes não dependam de instalação prévia pelo usuário e mantenham caminhos relativos à raiz da biblioteca.
- Documentar permissões e dependências de sistema no Linux.

**Non-Goals:**
- Suporte a macOS.
- Assinatura digital de binários ou auto-updater (questões #11 e #12 em aberto).

## Decisions

1. **Scripts nativos por plataforma**: PowerShell (`.ps1`) para Windows e Bash (`.sh`) para Linux, evitando dependências adicionais de build.
2. **Diretório `models/` vazio por padrão**: Respeita a regra de nunca instalar modelos volumosos automaticamente.
