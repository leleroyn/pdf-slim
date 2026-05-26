## ADDED Requirements

### Requirement: CLI argument parsing
The system SHALL accept command-line arguments for input files, output path, preset, quality, dimension, force, and JSON output.

#### Scenario: Single file input
- **WHEN** one PDF path is provided
- **THEN** that file is processed and output is written to `<name>.slim.pdf`

#### Scenario: Multiple file input
- **WHEN** multiple PDF paths are provided
- **THEN** each file is processed sequentially and a consolidated report is generated

#### Scenario: Output path specified
- **WHEN** `-o` / `--output` is provided
- **THEN** the output is written to the specified path instead of the default

### Requirement: Compression presets
The system SHALL provide three compression presets: `gentle`, `balanced`, and `aggressive`.

#### Scenario: Gentle preset
- **WHEN** `--preset gentle` is selected
- **THEN** JPEG quality is 85, no downsampling occurs, and PNG format is preserved

#### Scenario: Balanced preset (default)
- **WHEN** no preset is specified or `--preset balanced` is selected
- **THEN** JPEG quality is 70, max dimension is 1920, and lossy PNG conversion is enabled

#### Scenario: Aggressive preset
- **WHEN** `--preset aggressive` is selected
- **THEN** JPEG quality is 50, max dimension is 1280, and all PNG images are converted to JPEG

### Requirement: Force overwrite mode
The system SHALL support `--force` flag to overwrite the original file.

#### Scenario: Force overwrites original
- **WHEN** `--force` is set
- **THEN** the original file is replaced after a `.pdf-slim-bak` backup is created

#### Scenario: Default preserves original
- **WHEN** `--force` is not set
- **THEN** the original file is preserved and a new `.slim.pdf` file is created

### Requirement: Fine-grained override
The system SHALL allow `--image-quality` and `--max-dimension` to override preset defaults.

#### Scenario: Quality override
- **WHEN** `--image-quality 90` is provided with `aggressive` preset
- **THEN** image quality is 90 while other aggressive settings remain

### Requirement: Info mode
The system SHALL support `--info` flag for analysis without compression.

#### Scenario: Info mode runs analysis only
- **WHEN** `--info` is set
- **THEN** the PDF is analyzed, a size breakdown is reported, and no output file is written
