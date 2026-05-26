## 1. Project Scaffolding

- [x] 1.1 Initialize Rust project with `cargo init` and configure `Cargo.toml` with dependencies (clap, lopdf, image, rayon, tracing, serde, serde_json, anyhow)
- [x] 1.2 Create source directory structure: `src/compress/`, `src/analyze/`, `src/config/`, `src/utils/`
- [x] 1.3 Verify project builds with `cargo build`

## 2. CLI Interface

- [x] 2.1 Implement `Cli` struct with clap derive: input files, output, preset, image_quality, max_dimension, force, info, json, json_file
- [x] 2.2 Implement `Preset` enum (Gentle, Balanced, Aggressive) with ValueEnum derive
- [x] 2.3 Wire `main()` to parse args and dispatch to analysis or compression pipeline

## 3. Compression Configuration

- [x] 3.1 Define `CompressionConfig` struct (jpeg_quality, max_dimension, png_to_jpeg, compress_streams, remove_unused_fonts)
- [x] 3.2 Implement preset-to-config mapping: gentle (85/no-downscale), balanced (70/1920), aggressive (50/1280)
- [x] 3.3 Implement fine-grained override: CLI flags override preset defaults

## 4. Signature Detection

- [x] 4.1 Implement `has_digital_signature()` checking `/AcroForm /Fields` for Sig dictionaries
- [x] 4.2 Implement DocMDP detection via `/AcroForm /DOCMDP`
- [x] 4.3 Implement Permissions dictionary detection via `/Root /Permissions`
- [x] 4.4 Integrate signature check into pipeline: skip signed files with proper status reporting

## 5. Image Compression Core

- [x] 5.1 Implement image extraction: traverse Page → Resources → XObject → Image objects
- [x] 5.2 Implement image decoding for DCTDecode (JPEG) and CCITTFaxDecode formats
- [x] 5.3 Implement image downscaling with bilinear interpolation when exceeding max_dimension
- [x] 5.4 Implement JPEG re-encoding at configurable quality
- [x] 5.5 Implement PNG-to-JPEG conversion for lossy presets
- [x] 5.6 Implement image replacement: write re-encoded data back into PDF XObject
- [x] 5.7 Add rayon parallel processing for multiple images across pages
- [x] 5.8 Handle unsupported formats gracefully: skip JPEG2000, preserve CMYK

## 6. Stream Compression

- [x] 6.1 Scan all PDF streams for uncompressed content
- [x] 6.2 Apply Flate compression to uncompressed streams
- [x] 6.3 Track compressed stream count for reporting

## 7. File Output and Safety

- [x] 7.1 Implement default output: write to `<name>.slim.pdf`
- [x] 7.2 Implement `--force` mode: create `.pdf-slim-bak` backup, write to temp file, atomic rename
- [x] 7.3 Implement encrypted PDF detection and rejection

## 8. Compression Report

- [x] 8.1 Define JSON report structs with serde (Report, Summary, FileResult, Details, SignatureDetails)
- [x] 8.2 Implement human-readable terminal summary output
- [x] 8.3 Implement `--json` stdout output
- [x] 8.4 Implement `--json-file` file output
- [x] 8.5 Track per-file statistics: images processed, bytes before/after, reduction percentage

## 9. Info Mode

- [x] 9.1 Implement `--info` analysis: page count, image inventory, font list, size breakdown
- [x] 9.2 Display analysis report without modifying any files

## 10. Testing and Polish

- [x] 10.1 Add integration tests with sample PDFs for each preset
- [x] 10.2 Add test for digital signature detection and skip behavior
- [x] 10.3 Add test for `--force` overwrite with backup creation
- [x] 10.4 Add test for JSON report structure validation
- [x] 10.5 Test batch processing with multiple input files
- [x] 10.6 Verify release build size and add `Cargo.toml` metadata
