## ADDED Requirements

### Requirement: Digital signature detection
The system SHALL detect digital signatures in PDF files before compression.

#### Scenario: Signature in AcroForm fields
- **WHEN** a PDF contains `/AcroForm /Fields` with signature dictionaries
- **THEN** the signature is detected and the file is marked for skipping

#### Scenario: DocMDP certified signature
- **WHEN** a PDF contains `/AcroForm /DOCMDP` transform parameters
- **THEN** the signature is detected and the file is marked for skipping

#### Scenario: Permissions dictionary
- **WHEN** a PDF root contains a `/Permissions` dictionary
- **THEN** the signature is detected and the file is marked for skipping

#### Scenario: No signature present
- **WHEN** a PDF has no digital signature indicators
- **THEN** the file proceeds to normal compression processing

### Requirement: Signature skip reporting
The system SHALL report signature details in the compression report when a file is skipped.

#### Scenario: Skipped file includes signature info
- **WHEN** a file is skipped due to a digital signature
- **THEN** the report includes `skip_reason: "digital_signature_detected"` and signature type information

### Requirement: Signature detection does not modify PDF
The system SHALL perform signature detection as a read-only operation.

#### Scenario: Original file untouched on skip
- **WHEN** a signed PDF is detected
- **THEN** no write operation is performed on the original file and no output file is created
