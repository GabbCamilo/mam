# chapter-language-filter-and-separation Specification

## Purpose
TBD - created by archiving change implement-chapter-language-filter-and-separation. Update Purpose after archive.

## Requirements

### Requirement: Filter source chapters by language before archiving
The search and import view SHALL provide a `LANG` selector populated from the distinct languages present in the fetched chapters (plus an `All` option), filtering the visible chapter list and restricting bulk selection (`Selecionar Todas`) to the currently visible chapters.

#### Scenario: Filtering MangaDex chapters by PT
- **WHEN** the user selects a work with chapters in `PT` and `ENG` and chooses `PT` in the `LANG` filter
- **THEN** only `PT` chapters remain visible for selection and archiving.

### Requirement: Separate archived chapters by language directory and persist language metadata
The database `capitulos` table SHALL store `idioma`, and archiving a chapter with a language code SHALL place the `.cbz` file inside `Manga/<Obra>/<LANG>/Capitulo_XXXX.cbz` so multiple languages of the same chapter do not overwrite each other.

#### Scenario: Archiving Chapter 1 in PT and ENG
- **WHEN** the user archives Chapter 1 in `PT` and Chapter 1 in `ENG` for the same manga
- **THEN** both `.cbz` files are preserved in separate language subdirectories (`PT/` and `ENG/`) and indexed with their respective `idioma` values.

### Requirement: Library language selector, directory spacing, and language-scoped reader navigation
The library work view SHALL provide a `LANG` selector when chapters span multiple languages, render clear visual spacing between language directories when viewing all languages, and pass only chapters matching the active chapter's language to `CbzReader` so next/previous navigation stays within the selected language.

#### Scenario: Navigating next chapter in reader with multiple languages archived
- **WHEN** the user opens a `PT` chapter of a manga that also has `ENG` chapters
- **THEN** `CbzReader` advances only through `PT` chapters in ascending chapter order.
