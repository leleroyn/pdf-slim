# pdf-slim

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/leleroyn/pdf-slim/actions/workflows/release.yml/badge.svg)](https://github.com/leleroyn/pdf-slim/actions/workflows/release.yml)

A fast, lightweight PDF compression tool written in Rust. Reduces PDF file size by re-encoding images at lower quality, downsampling large images, and compressing uncompressed streams — while preserving document layout and readability.

## Features

- **Image compression** — Re-encode JPEG images at configurable quality, downsample oversized images
- **Stream compression** — Apply Flate compression to uncompressed PDF streams
- **FlateDecode + DCTDecode support** — Handles zlib-wrapped JPEG images common in scanned PDFs
- **Digital signature detection** — Automatically skips signed PDFs to avoid invalidating signatures
- **Encrypted PDF rejection** — Refuses to process encrypted files
- **Force mode with backup** — Overwrite original file with automatic `.pdf-slim-bak` backup
- **Info mode** — Analyze PDF structure without modifying any files
- **JSON reporting** — Output detailed compression reports to stdout or file
- **Batch processing** — Compress multiple files in a single command
- **Three presets** — Gentle, Balanced, Aggressive compression levels
- **Parallel processing** — Multi-threaded image compression via rayon

## Installation

### From Source

Requires [Rust 1.70+](https://rustup.rs/) and a C toolchain.

```bash
git clone https://github.com/leleroyn/pdf-slim.git
cd pdf-slim
cargo build --release
cp target/release/pdf-slim ~/.local/bin/
```

### From GitHub Releases

Download the pre-built binary for your platform from the [Releases](https://github.com/leleroyn/pdf-slim/releases) page.

```bash
# Linux
wget https://github.com/leleroyn/pdf-slim/releases/download/v0.1.0/pdf-slim-x86_64-unknown-linux-gnu.tar.gz
tar xzf pdf-slim-x86_64-unknown-linux-gnu.tar.gz
chmod +x pdf-slim
```

## Usage

```
pdf-slim [OPTIONS] <INPUT>...
```

### Basic

```bash
# Compress a single file (creates input.slim.pdf)
pdf-slim document.pdf

# Compress and overwrite original (with backup)
pdf-slim --force document.pdf

# Analyze without compressing
pdf-slim --info document.pdf
```

### Presets

| Preset | JPEG Quality | Max Dimension | PNG→JPEG | Use Case |
|--------|-------------|---------------|----------|----------|
| `gentle` | 85 | No downsampling | No | Minimal quality loss |
| `balanced` | 70 | 1920px | Yes | Default, good tradeoff |
| `aggressive` | 50 | 1280px | Yes | Maximum compression |

```bash
pdf-slim --preset gentle document.pdf
pdf-slim --preset aggressive document.pdf
```

### Fine-tuning

```bash
# Custom JPEG quality (1-100)
pdf-slim --image-quality 90 document.pdf

# Custom max image dimension
pdf-slim --max-dimension 2560 document.pdf

# Combine preset with overrides
pdf-slim --preset gentle --image-quality 75 --max-dimension 1600 document.pdf
```

### Batch Processing

```bash
# Compress multiple files
pdf-slim file1.pdf file2.pdf file3.pdf

# Compress all PDFs in a directory
pdf-slim *.pdf
```

### JSON Reporting

```bash
# Output report to stdout
pdf-slim --json document.pdf

# Write report to file
pdf-slim --json-file report.json document.pdf
```

## Examples

### Compress a scanned document

```bash
# Scanned PDFs often have large images — aggressive preset works well
pdf-slim --preset aggressive --force scanned-documents.pdf
# Original: 45 MB  →  Compressed: 8.2 MB  (82% reduction)
```

### Compress with custom settings

```bash
# High quality, moderate downsampling
pdf-slim --preset gentle --image-quality 80 --max-dimension 2048 photo-collection.pdf
```

### Analyze before compressing

```bash
# Check what's inside the PDF
pdf-slim --info large-file.pdf
# Shows: page count, image inventory, font list, size breakdown
```

### CI/CD integration

```bash
# Compress and get JSON report for logging
pdf-slim --json --force document.pdf | jq '.summary.total_reduction_percent'
```

## Architecture

```
pdf-slim
├── CLI (clap)          Argument parsing, preset selection
├── Config (presets)    Compression settings from preset + overrides
├── Analyze             PDF analysis, signature detection, info mode
│   ├── report          JSON report structs
│   └── info            PDF structure inspection
├── Compress            Compression pipeline
│   ├── images          Image extraction, decoding, re-encoding
│   └── streams         Stream Flate compression
└── Utils               Path handling, formatting, summary output
```

### Compression Pipeline

1. **Load PDF** — Parse document structure via lopdf
2. **Safety checks** — Reject encrypted files, skip signed files
3. **Image compression** — Extract → Decode → Downscale → Re-encode as JPEG
4. **Stream compression** — Apply Flate to uncompressed streams
5. **Output** — Write to `.slim.pdf` or overwrite with backup

## Testing

```bash
cargo test
cargo test --release
```

## Known Limitations

- PDFs with damaged cross-reference tables may fail to load — repair with `qpdf --repair input.pdf -o fixed.pdf`
- JPEG2000 (JPXDecode) images are skipped and preserved as-is
- CMYK and ICCBased color space images are skipped to avoid color shift
- Inline images (non-XObject) are not processed

## Contributing

Issues and pull requests are welcome. Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit changes (`git commit -m 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request

## License

MIT — See [LICENSE](LICENSE) for details.
