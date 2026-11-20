# async-download-toast-and-live-log Specification

## Purpose
TBD - created by archiving change implement-async-download-toast-and-live-log. Update Purpose after archive.

## Requirements

### Requirement: Non-blocking batch archiving with temporary clickable popup
Clicking `Arquivar Selecionados` SHALL start the archiving operation in the background, immediately display a temporary clickable popup indicating that the batch is downloading, and navigate to the `Log de Downloads` tab when the user clicks the popup.

#### Scenario: User clicks the temporary download popup
- **WHEN** the user clicks `Arquivar Selecionados` and clicks the temporary popup that appears
- **THEN** the application switches to the `downloads` tab showing the live batch progress.

### Requirement: Live silent counter updates and batch progress in Log de Downloads
While one or more archiving operations are in progress, the frontend SHALL periodically poll operation status and download history so the `Log de Downloads (N)` tab counter increments silently as each chapter completes and the `Log de Downloads` view displays live batch progress (`capitulos_processados / total_capitulos`).

#### Scenario: Chapters complete sequentially in background
- **WHEN** chapters finish downloading one by one in the background task
- **THEN** the `Log de Downloads (N)` counter increments silently for each completed chapter and the batch progress bar updates until completion.
