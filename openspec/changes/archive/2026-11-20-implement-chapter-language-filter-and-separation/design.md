# Design Técnico

## Camadas afetadas
1. **Database (`backend/database`)**:
   - Migração `002_capitulos_idioma.sql`: `ALTER TABLE capitulos ADD COLUMN idioma TEXT;`.
   - `Capitulo` em `models.rs` ganha `pub idioma: Option<String>` com `#[serde(default)]`.
   - `operations.rs`: novo método `criar_capitulo_com_idioma(...)` (e `criar_capitulo(...)` delega com `None`) + `listar_capitulos_obra` lê `idioma`.
2. **Source Integration (`backend/source-integration/src/lib.rs`)**:
   - `arquivar_para_biblioteca_com_capa` calcula `subpasta_idioma` quando `cap.idioma` não for vazio (`Manga/<Obra>/<LANG>`), salvando `Capitulo_XXXX.cbz` dentro dela e registrando `cap.idioma` via `criar_capitulo_com_idioma`.
3. **Frontend (`backend/frontend/src/api.ts`, `App.tsx`)**:
   - Interface `Capitulo` recebe `idioma?: string | null`.
   - Aba **Fontes & Importação**: estado `filtroIdiomaFonte`, seletor `LANG` dinâmico e `capitulosFonteFiltrados` usado na lista e em `alternarSelecionarTodosCapitulos`.
   - Aba **Biblioteca**: estado `filtroIdiomaBiblioteca`, seletor `LANG`, agrupamento com espaçamento entre diretórios de idiomas quando `filtroIdiomaBiblioteca === 'TODOS'` e houver mais de um idioma.
   - **Leitor (`CbzReader`)**: `capitulosDisponiveis` recebe somente capítulos do mesmo `idioma` (ou mesmo diretório) do `capituloAtivo`.
