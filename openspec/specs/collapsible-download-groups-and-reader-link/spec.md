# collapsible-download-groups-and-reader-link Specification

## Purpose
TBD - created by archiving change implement-collapsible-download-groups-and-reader-link. Update Purpose after archive.

## Requirements

### Requirement: Collapsible batch groups in Download Log
The Download Log view SHALL group downloaded chapters by their download batch (`operacao_id`, with fallback grouping for legacy entries) into collapsible cards that toggle between showing and hiding their downloaded chapters when clicked.

#### Scenario: Toggling a completed batch in Download Log
- **WHEN** the user clicks a completed batch header in `Log de Downloads`
- **THEN** its downloaded chapter list expands; clicking the header again collapses it.

### Requirement: Open chapter directly in reader from Download Log
Each downloaded chapter entry inside a Download Log group SHALL allow opening the chapter directly in `CbzReader` without navigating to the Library first.

#### Scenario: Clicking Ler on a chapter in Download Log
- **WHEN** the user clicks `Ler` on a chapter inside an expanded Download Log group
- **THEN** the application loads the work details and opens `CbzReader` on that chapter.
