# deletion-log-volume-builder-and-double-page Specification

## Purpose
TBD - created by archiving change implement-deletion-log-volume-builder-and-double-page. Update Purpose after archive.

## Requirements

### Requirement: Deletion log and work deletion
The system SHALL allow deleting an entire work from the library (catalog and files) and SHALL record both chapter deletions and work deletions in `historico_operacoes` (`tipo = 'exclusao'`), displaying them in the UI deletion log.

#### Scenario: Deleting a chapter or work logs the deletion
- **WHEN** the user deletes a chapter or a work in the Library
- **THEN** the item is removed and an entry appears in the Deletion Log with name, path, and timestamp.

### Requirement: Volume creation from selected chapters and renaming of volumes and chapters
The Library Volumes view SHALL allow selecting chapters to form a volume with an order number and name, renaming existing volumes, and renaming the title of any downloaded chapter.

#### Scenario: Forming and renaming a volume
- **WHEN** the user selects chapters, enters order number and volume name, and submits
- **THEN** the volume is created containing those chapters and can subsequently be renamed.

### Requirement: Equal 50/50 double-page spread in CbzReader
In `dupla` layout mode, `CbzReader` SHALL render the two pages in equal 50%/50% columns with `object-fit: contain` so differing image resolutions do not cause mismatched page sizes.

#### Scenario: Viewing two pages of different resolutions in Double mode
- **WHEN** `modo === 'dupla'` is active
- **THEN** both pages occupy equal 50% width columns side by side.
