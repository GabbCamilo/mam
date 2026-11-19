# Design — implement-archive-cbz

## Context

O banco de dados (Fase 3) e o gerenciamento de capítulos (Fase 4) já estão implementados. Agora é necessário o Archive Manager para criar arquivos CBZ a partir de imagens e validar CBZ existentes. O CBZ é um arquivo ZIP contendo imagens sequenciais numeradas e ordenadas.

## Goals / Non-Goals

**Goals:**
- Criar arquivos CBZ a partir de imagens em um diretório
- Ler arquivos CBZ existentes (listar páginas, quantidade, metadados)
- Validar integridade de arquivos CBZ (ZIP válido, imagens válidas)
- Detectar páginas ausentes (numeração não sequencial)
- Gravação segura via arquivos temporários + rename atômico
- Integrar com o banco de dados para registrar capítulos criados

**Non-Goals:**
- Conversão de formatos de imagem (JPG/PNG/WebP) — apenas empacotamento
- Download de capítulos de fontes externas
- Leitura visual de páginas (fase do leitor integrado)
- OCR ou processamento de IA sobre CBZ (fase AI Engine)

## Decisions

1. **Formato CBZ = ZIP** — arquivo ZIP com extensão .cbz, contendo imagens nomeadas sequencialmente (001.jpg, 002.jpg, etc.)
2. **Criação: crate `zip`** — ZipWriter para criar arquivos, adicionando imagens em ordem
3. **Leitura: crate `zip`** — ZipArchive para listar entradas sem extração permanente
4. **Compressão: Deflate (padrão ZIP)** — sem compressão adicional específica
5. **Escrita segura: arquivo temporário + fs::rename** — cria em temp/.cbz.tmp, depois renomeia para o destino final (operação atômica no mesmo filesystem)
6. **Ordenação de páginas: ordenação lexicográfica por nome de arquivo** — imagens nomeadas 001, 002, etc.
7. **Validação: tentar abrir como ZIP + verificar extensões de imagem** — arquivo corrompido é rejeitado
8. **Integração com banco: registrar capítulo criado via mam-database** — após criar CBZ com sucesso, registrar no banco

## Arquitetura

```
backend/
├── archive/              # mam-archive — geração e leitura de CBZ
│   ├── src/
│   │   ├── lib.rs        # ArchiveManager ponto de entrada
│   │   ├── criacao.rs    # geração de CBZ a partir de imagens
│   │   ├── leitura.rs    # leitura e validação de CBZ existentes
│   │   ├── validacao.rs  # detecção de páginas ausentes, integridade
│   │   └── operacoes_seguras.rs  # escrita com temp + rename atômico
└── src-tauri/            # comandos Tauri expostos ao frontend
```

## Fluxo de criação de CBZ

```
imagens/ (001.jpg, 002.jpg, ..., 020.jpg)
      ↓
criar_cbz(destino.cbz, imagens_dir)
      ↓
1. Listar imagens ordenadas
2. Criar arquivo temp destino.cbz.tmp
3. Para cada imagem: adicionar ao ZIP com nome sequencial
4. Finalizar ZIP, fechar arquivo
5. fs::rename(temp, destino) — atômico
6. Registrar capítulo no banco (via mam-database)
7. Calcular SHA-256, registrar integridade
```

## Regras obrigatórias

- CBZ originais nunca modificados (criação gera novos arquivos)
- Geração utiliza arquivos temporários e operações de escrita seguras
- Nunca excluir arquivos de origem (imagens originais intactas)
- Caminhos relativos à raiz da biblioteca
- SHA-256 para integridade do CBZ gerado
- Páginas numeradas e ordenadas

## Risks / Trade-offs

- [CBZ grande] arquivos com muitas imagens podem consumir memória; mitigado com streaming via ZipWriter (não carrega tudo na memória)
- [Nomeação de imagens] imagens sem numeração sequencial podem resultar em ordem incorreta; mitigado com ordenação lexicográfica e documentação
- [Atomicidade] fs::rename não é atômico em diferentes filesystems; mitigado garantindo que temp e destino estejam no mesmo diretório

## Migration Plan

1. Criar crate `mam-archive` no workspace
2. Implementar criacao, leitura, validacao, operacoes_seguras
3. Adicionar comandos Tauri em `mam-interface` e `src-tauri`
4. Validar com `cargo build` e `cargo test`
5. Arquivar change

## Open Questions

- Extensão de imagem suportada: JPG, PNG, WebP? (decisão: todas as comuns)
- Nível de compressão ZIP: padrão vs máximo? (decisão: padrão, velocidade > tamanho)
