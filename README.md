# Manga Archive Manager (MAM)

Aplicação desktop multiplataforma (Windows e Linux) para gerenciamento, arquivamento, organização, leitura, transferência e processamento inteligente de mangás e quadrinhos digitais.

## Objetivo

Proporcionar uma biblioteca digital independente do computador, armazenada em SSD externo, acessível em diferentes sistemas operacionais sem depender de servidores ou bancos de dados remotos.

## Stack

- **Tauri 2** — aplicação desktop
- **React + TypeScript** — frontend
- **Rust** — backend (monólito modular)
- **SQLite** — banco de dados local
- **Python** — Manga AI Engine (processo auxiliar)
- **ONNX Runtime / PyTorch** — inferência local

## Plataformas

Windows e Linux. macOS está fora do escopo.

## Metodologia

OpenSpec (Spec-Driven Development) com schema `spec-driven`.

## Documentação

Consulte `docs/` para visão do produto, escopo, arquitetura, requisitos e glossário. Consulte `openspec/` para especificações funcionais e changes em andamento.

## Licença

Privado — uso interno.
