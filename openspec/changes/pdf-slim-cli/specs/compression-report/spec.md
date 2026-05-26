## ADDED Requirements

### Requirement: JSON report structure
The system SHALL generate a structured JSON report with version, timestamp, summary, and per-file details.

#### Scenario: Report contains summary
- **WHEN** processing completes
- **THEN** the report includes `summary` with total files, compressed count, skipped count, error count, total sizes, reduction percentage, and elapsed time

#### Scenario: Report contains per-file details
- **WHEN** each file is processed
- **THEN** the `files` array includes an entry with input path, output path, status, sizes, reduction percentage, and processing details

### Requirement: File status classification
The system SHALL classify each file with one of three statuses: `compressed`, `skipped`, or `error`.

#### Scenario: Successful compression
- **WHEN** a file is compressed successfully
- **THEN** status is `compressed` with output path and reduction percentage

#### Scenario: Skipped file
- **WHEN** a file is skipped (signature detected, output exists)
- **THEN** status is `skipped` with `skip_reason` explaining why

#### Scenario: Error file
- **WHEN** a file fails to process
- **THEN** status is `error` with an `error` message describing the failure

### Requirement: Image processing details
The system SHALL report image compression statistics per file.

#### Scenario: Image stats reported
- **WHEN** images are processed
- **THEN** the `details` object includes `images_processed`, `images_original_bytes`, and `images_compressed_bytes`

### Requirement: JSON output modes
The system SHALL support two JSON output modes: stdout and file.

#### Scenario: JSON to stdout
- **WHEN** `--json` flag is set
- **THEN** the full JSON report is written to stdout and human-readable output is suppressed

#### Scenario: JSON to file
- **WHEN** `--json-file <path>` is provided
- **THEN** the full JSON report is written to the specified file path

#### Scenario: Default human-readable output
- **WHEN** neither `--json` nor `--json-file` is set
- **THEN** a concise human-readable summary is printed to stdout
