## ADDED Requirements

### Requirement: PDF object tree traversal
The system SHALL traverse the PDF object tree to access all pages, resources, and embedded objects.

#### Scenario: Successful PDF load
- **WHEN** a valid PDF file is provided
- **THEN** the system loads the document and reports the total page count

#### Scenario: Corrupted PDF rejected
- **WHEN** a corrupted or invalid PDF file is provided
- **THEN** the system reports an error with the file status set to "error" and continues processing remaining files

### Requirement: Compression pipeline execution
The system SHALL execute compression stages sequentially: image re-encoding, stream compression, and object cleanup.

#### Scenario: Full pipeline completes
- **WHEN** a compressible PDF is processed
- **THEN** all compression stages execute in order and the output file is written

### Requirement: Safe file write
The system SHALL write output to a temporary file first, then rename to the target path.

#### Scenario: Atomic replace on force overwrite
- **WHEN** `--force` is set and output would overwrite the input file
- **THEN** a `.pdf-slim-bak` backup is created, the temp file is renamed to the original path, and the original content is replaced

#### Scenario: Default non-destructive output
- **WHEN** `--force` is not set
- **THEN** the output is written to `<input_name>.slim.pdf` and the original file remains unchanged

### Requirement: Encrypted PDF handling
The system SHALL detect encrypted PDFs and skip processing them.

#### Scenario: Encrypted PDF skipped
- **WHEN** an encrypted PDF is provided
- **THEN** the file status is "error" with a message indicating encryption was detected
