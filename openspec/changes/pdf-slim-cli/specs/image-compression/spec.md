## ADDED Requirements

### Requirement: Image extraction from PDF
The system SHALL extract all embedded images from PDF XObject resources.

#### Scenario: Images found in page resources
- **WHEN** a PDF page contains image XObjects
- **THEN** each image is decoded with its color space, dimensions, and decode parameters recorded

#### Scenario: No images in PDF
- **WHEN** a PDF contains no image XObjects
- **THEN** the image compression stage is skipped and processing continues

### Requirement: Image downscaling
The system SHALL downscale images exceeding the maximum dimension threshold.

#### Scenario: Image exceeds max dimension
- **WHEN** an image width or height exceeds `--max-dimension`
- **THEN** the image is scaled down proportionally to fit within the threshold using bilinear interpolation

#### Scenario: Image within bounds
- **WHEN** an image fits within `--max-dimension`
- **THEN** the image is not scaled

### Requirement: JPEG re-encoding
The system SHALL re-encode images as JPEG at the specified quality level.

#### Scenario: JPEG quality applied
- **WHEN** image quality is set (via preset or `--image-quality`)
- **THEN** all re-encoded JPEG images use the specified quality value

#### Scenario: PNG converted to JPEG
- **WHEN** a lossy preset is active and a PNG image has more than 2 color channels
- **THEN** the PNG is converted to JPEG encoding

#### Scenario: Grayscale images preserved
- **WHEN** a PNG image is 1-bit or grayscale with fewer than 2 channels
- **THEN** the image retains its original encoding format

### Requirement: Parallel image processing
The system SHALL process multiple images concurrently using available CPU cores.

#### Scenario: Multiple images processed in parallel
- **WHEN** a PDF page contains multiple images
- **THEN** images are processed concurrently and all replacements are applied to the document

### Requirement: Unsupported image format handling
The system SHALL gracefully handle image formats it cannot re-encode.

#### Scenario: JPEG2000 image encountered
- **WHEN** an image uses JPXDecode (JPEG2000) compression
- **THEN** the image is skipped, logged in the report, and processing continues with remaining images

#### Scenario: CMYK image encountered
- **WHEN** an image uses CMYK color space
- **THEN** the image is skipped during re-encoding and the original data is preserved
