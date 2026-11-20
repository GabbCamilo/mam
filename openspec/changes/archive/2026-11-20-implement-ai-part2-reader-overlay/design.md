# Design — implement-ai-part2-reader-overlay

## Context

Parte 2 de 3 da evolução do Manga AI Engine: integração leitor ↔ AI Engine ↔ SQLite.

## Decisions

1. **Arquivo temporário isolado por UUID**: A página é decodificada de `PaginaExtraida` para um arquivo temporário em `std::env::temp_dir()`, passada por caminho ao processo Python (evitando Base64 gigante no JSON-RPC), e removida imediatamente após a resposta.
2. **Camada de sobreposição CSS percentual**: `left: x*100%`, `top: y*100%`, `width: w*100%`, `height: h*100%` dentro de um container `position: relative` envolvendo a `<img>`.
