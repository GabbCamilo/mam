# Spec: Central Manga Link Discovery and Import Modal Redesign

## ADDED Requirements

### Requirement: Redesigned two-mode Import Modal
The Import modal SHALL separate **Central Work Page (Batch)** discovery from **Single Chapter / Manual Image List** import so users can paste a single work URL (or optional page HTML) without chapter-level fields cluttering the flow.

#### Scenario: Discovering a full series from its central URL
- **WHEN** the user opens the Import modal, selects the Central Work Page tab, enters `https://mangafire.to/title/jw2-medaka-boxx` (or `https://mangalivre.to/manga/chainsaw-man-pt-br/`), and submits
- **THEN** the application switches to `Fontes & Importação` showing the work metadata and all discovered chapters ready for language filtering, bulk selection, and background archiving.

### Requirement: Structural chapter-link analyzer with site adapters and optional AI classification
The backend SHALL extract chapter links, numbers, titles, and languages from central work pages by combining site-pattern adapters (`MangaFire`, `MangaLivre`), repeating DOM cluster heuristics, embedded JSON state inspection, and optional local `ai-engine` link-cluster classification (without bypassing anti-bot or DRM).

#### Scenario: Parsing repeating chapter link clusters
- **WHEN** a central manga page contains links following `/title/<slug>/chapter/<upload_code>` or `/manga/<slug>/capitulo-<N>/`
- **THEN** the analyzer deduplicates, normalizes, extracts chapter numbers and language tags, and sorts chapters in ascending order.
