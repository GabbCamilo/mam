# Design — implement-ux-improvements-and-longstrip

## Decisions

1. **Retrocompatibilidade de DTOs**: `idioma: Option<String>` e `capa_url: Option<String>` usam `#[serde(default)]`.
2. **Long strip progressivo**: No modo `vertical` (`Long strip`), o `CbzReader` carrega todas as páginas (`0..totalPaginas`) sob demanda para rolagem vertical contínua.
3. **Modal `+ Importar URL`**: Movido para modal flutuante acessível pelo botão primário do topo.
