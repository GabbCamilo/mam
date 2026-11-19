# Empacotamento e Distribuição — Manga Archive Manager

## Plataformas

- Windows: pacote portátil ZIP
- Linux: AppImage (formato portátil prioritário), TAR.GZ (alternativa)

## Conteúdo dos pacotes

### MAM Core
- Interface (React + TypeScript via Tauri)
- Backend (Rust)
- SQLite
- Leitor CBZ
- Biblioteca
- Armazenamento

### MAM AI Runtime
- Executável Python empacotado
- Bibliotecas de inferência (ONNX Runtime, PyTorch quando necessário)
- Dependências de processamento

### MAM AI Models
- Modelos instaláveis individualmente

## Requisitos

- Pacotes produzidos separadamente por sistema operacional e arquitetura de processador (x86-64)
- Usuário não precisa instalar Python, Node.js ou banco de dados
- Executáveis auxiliares compatíveis com a distribuição-alvo (Linux)
- Dependências e permissões documentadas (Linux)

## Distribuição

- Não exigir instalação convencional obrigatória
- Permitir execução a partir de diferentes unidades de armazenamento
- Preservar portabilidade de dados e configurações

## Gerenciamento de atualizações

A definir (ponte técnico pendente — questão #11 da Etapa D)
