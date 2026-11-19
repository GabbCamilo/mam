# Escopo do Projeto — Manga Archive Manager

## Dentro do escopo (MVP)

- Biblioteca digital de mangás e quadrinhos (obra > volume > capítulo .cbz)
- Gerenciamento de armazenamento (identificação de bibliotecas, unidades, espaço, duplicatas, integridade SHA-256)
- Pesquisa e integração com fontes autorizadas (somente arquivamento autorizado)
- Geração e leitura de arquivos CBZ
- Leitor integrado (navegação, zoom, modo dupla página, favoritos, marcadores, histórico)
- Transferência integral, sincronização e transferência seletiva
- Manga AI Engine local (OCR, detecção de balões, tradução, revisão)
- Gerenciamento opcional de modelos de IA
- Cache multicamadas (L1 memória, L2 disco, L3 processamento, L4 modelos, L5 original)
- Banco de dados SQLite local
- Empacotamento portátil (ZIP Windows, AppImage/TAR.GZ Linux)

## Fora do escopo (MVP)

- macOS (não suportado)
- Monitoramento automático de novas disponibilizações (funcionalidade futura, requisitos não definidos)
- Extensões/plugins de terceiros (reavaliar após estabilização)
- APIs de IA em nuvem (processamento exclusivamente local)
- Contornar DRM, paywalls, autenticação ou controles de acesso
- Servidor remoto ou banco de dados externo
- Arquitetura de microsserviços

## Regras obrigatórias

- Nunca excluir automaticamente arquivos da origem
- Caminhos relativos à raiz da biblioteca
- Preservar CBZ originais durante processamento de IA
- Processamento de IA exclusivamente local
- Não instalar modelos automaticamente
