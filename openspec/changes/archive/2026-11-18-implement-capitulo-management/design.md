# Design — implement-capitulo-management

## Context

O banco de dados SQLite (Fase 3) já está implementado. Agora é necessário preenchê-lo com capítulos detectados a partir de arquivos .cbz no filesystem. O ChapterManager deve varrer a estrutura da biblioteca (Manga/ > Obra > [Volume/] > Capítulo.cbz), extrair metadados de cada CBZ, calcular hash SHA-256 e registrar no banco.

## Goals / Non-Goals

**Goals:**
- Detectar arquivos .cbz na estrutura hierárquica da biblioteca
- Extrair lista de páginas ordenadas de cada CBZ
- Calcular hash SHA-256 do arquivo .cbz para integridade
- Registrar capítulos no banco SQLite (via mam-database)
- Expor comandos Tauri para o frontend (listar capítulos, verificar integridade)
- Manter caminhos relativos à raiz da biblioteca

**Non-Goals:**
- Leitura de páginas para visualização (fase do leitor integrado)
- Conversão de imagens para CBZ (fase Archive Manager)
- Download de capítulos de fontes externas (fonte integration)
- Modificação de CBZ originais

## Decisions

1. **Detecção: walkdir** — varredura recursiva do diretório `Manga/` identificando arquivos com extensão `.cbz`.
2. **Leitura CBZ: crate `zip`** — CBZ é um arquivo ZIP; `zip` permite listar entradas sem extração permanente. Alternativa `cbz-rs` rejeitada por ser menos mantida.
3. **Ordenação de páginas: ordenação lexicográfica por nome de arquivo** — páginas nomeadas `001.jpg`, `002.jpg`, etc. são ordenadas naturalmente pelo nome.
4. **Hash SHA-256: crate `sha2`** — cálculo do hash do arquivo .cbz completo para integridade.
5. **Estrutura híbrida de volumes: detecção por pasta** — se o CBZ está dentro de uma pasta `Volume XXX`, associa a esse volume; caso contrário, associa diretamente à obra.
6. **Detecção somente leitura** — o ChapterManager nunca escreve nos arquivos .cbz, apenas lê para extrair metadados.
7. **Registro no banco via mam-database** — usa as operações `criar_obra`, `criar_capitulo`, `registrar_integridade` já implementadas.

## Arquitetura

```
backend/
├── chapter/              # mam-chapter — detecção e parsing de capítulos
│   ├── src/
│   │   ├── lib.rs        # ChapterManager ponto de entrada
│   │   ├── detector.rs   # varredura walkdir de arquivos .cbz
│   │   ├── parser.rs     # leitura de páginas do CBZ via crate zip
│   │   ├── integrity.rs  # hash SHA-256 do arquivo
│   │   └── operations.rs # registro no banco via mam-database
├── storage/              # mam-storage — identificação de bibliotecas e unidades
│   └── src/lib.rs        # StorageManager
└── src-tauri/            # comandos Tauri expostos ao frontend
```

## Fluxo de detecção de capítulo

```
raiz_biblioteca/Manga/
  └── Obra A/
      ├── Capitulo 0001.cbz          -> obra_id=ObraA, volume_id=None
      └── Volume 001/
          └── Capitulo 0002.cbz      -> obra_id=ObraA, volume_id=Vol001
```

1. Varrer `Manga/` recursivamente por arquivos `*.cbz`
2. Para cada .cbz:
   a. Calcular caminho relativo da raiz
   b. Calcular hash SHA-256
   c. Extrair lista de páginas (nomes ordenados)
   d. Determinar obra (pasta pai direta ou avô se dentro de volume)
   e. Determinar volume (pasta intermediária se existir)
   f. Registrar obra (se não existir), volume (se aplicável), capítulo no banco
   g. Registrar integridade SHA-256

## Regras obrigatórias

- Caminhos relativos à raiz da biblioteca (não letra de unidade)
- SHA-256 para integridade de cada capítulo
- CBZ originais nunca modificados (leitura apenas)
- Não carregar todas as páginas simultaneamente (apenas listar nomes)
- Transações curtas no banco

## Risks / Trade-offs

- [Ordenação lexicográfica] `Capítulo 10.cbz` vem antes de `Capítulo 2.cbz` em ordenação lexicográfica pura; mitigado com extração de número do nome quando possível.
- [CBZ corrompido] arquivo .cbz inválido ou ZIP corrompido; mitigado com tratamento de erro e registro como `estado_integridade = invalido`.
- [Performance] varredura de bibliotecas grandes pode ser lenta; mitigado com detecção assíncrona e progresso via eventos Tauri.

## Migration Plan

1. Criar crate `mam-chapter` no workspace
2. Implementar detector, parser, integrity, operations
3. Implementar StorageManager em `mam-storage`
4. Adicionar comandos Tauri em `mam-interface` e `src-tauri`
5. Validar com `cargo build` e `cargo test`
6. Arquivar change

## Open Questions

- Nomeação de obras: usar nome da pasta ou metadados internos do CBZ? (decisão: nome da pasta por simplicidade)
- Extração de número do capítulo do nome do arquivo (regex) vs número sequencial? (avaliar)
