pub mod images;
pub mod streams;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lopdf::Document;

use crate::analyze::report::{
    CompressionDetails, FileResult, FileStatus, Report, SkipReason, Summary,
};
use crate::compress::images::compress_images;
use crate::compress::streams::compress_streams;
use crate::config::presets::CompressionConfig;
use crate::utils::{backup_path, default_output_path, reduction_percent};

/// Process a single PDF file through the compression pipeline.
pub fn process_file(
    input_path: &Path,
    config: &CompressionConfig,
    force: bool,
) -> Result<FileResult> {
    // Load document
    let load_doc = |p: &Path| -> Result<Document> {
        Document::load(p).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("cross-reference") || msg.contains("cross reference") {
                anyhow::anyhow!(
                    "PDF has a damaged or non-standard cross-reference table: {}\n\
                     Try repairing with: qpdf --repair {} - > {}_fixed.pdf",
                    msg,
                    p.display(),
                    p.display()
                )
            } else if msg.contains("trailer") || msg.contains("Trailer") {
                anyhow::anyhow!(
                    "PDF has a missing or invalid file trailer: {}\n\
                     Try repairing with: qpdf --repair {} - > {}_fixed.pdf",
                    msg,
                    p.display(),
                    p.display()
                )
            } else {
                anyhow::anyhow!("Failed to load PDF: {}", msg)
            }
        })
    };
    let original_size = std::fs::metadata(input_path)
        .with_context(|| format!("Failed to read metadata: {}", input_path.display()))?
        .len();

    let mut doc = load_doc(input_path)?;

    // Check encryption
    if doc.is_encrypted() {
        return Ok(FileResult {
            input_path: input_path.display().to_string(),
            output_path: None,
            status: FileStatus::Error,
            original_size_bytes: original_size,
            output_size_bytes: None,
            reduction_percent: 0.0,
            pages: Some(doc.get_pages().len()),
            skip_reason: None,
            signature_details: None,
            error: Some("PDF is encrypted".to_string()),
            details: None,
        });
    }

    // Check for digital signatures
    if let Some(sig_details) = crate::analyze::check_digital_signature(&doc) {
        return Ok(FileResult {
            input_path: input_path.display().to_string(),
            output_path: None,
            status: FileStatus::Skipped,
            original_size_bytes: original_size,
            output_size_bytes: None,
            reduction_percent: 0.0,
            pages: Some(doc.get_pages().len()),
            skip_reason: Some(SkipReason::DigitalSignatureDetected),
            signature_details: Some(sig_details),
            error: None,
            details: None,
        });
    }

    let page_count = doc.get_pages().len();

    // Stage 1: Image compression
    let image_stats = compress_images(&mut doc, config)?;

    // Stage 2: Stream compression
    let stream_stats = compress_streams(&mut doc)?;

    // Determine output path
    let output_path = if force {
        input_path.to_path_buf()
    } else {
        default_output_path(input_path)
    };

    // If force mode, create backup
    if force {
        let bak = backup_path(input_path);
        std::fs::copy(input_path, &bak)
            .with_context(|| format!("Failed to create backup: {}", bak.display()))?;
    }

    // Write to temp file first
    let temp_path = output_path.with_extension(".tmp");
    doc.save(&temp_path)
        .with_context(|| format!("Failed to save PDF: {}", temp_path.display()))?;

    // Compare sizes — only keep the result if it's actually smaller
    let temp_size = std::fs::metadata(&temp_path)
        .with_context(|| format!("Failed to read temp file metadata"))?
        .len();

    if temp_size >= original_size {
        // No benefit — remove temp file and leave original untouched
        let _ = std::fs::remove_file(&temp_path);
        if force {
            let _ = std::fs::remove_file(&backup_path(input_path));
        }
        return Ok(FileResult {
            input_path: input_path.display().to_string(),
            output_path: None,
            status: FileStatus::Skipped,
            original_size_bytes: original_size,
            output_size_bytes: None,
            reduction_percent: 0.0,
            pages: Some(page_count),
            skip_reason: Some(SkipReason::NoCompressionBenefit),
            signature_details: None,
            error: None,
            details: Some(CompressionDetails {
                images_processed: Some(image_stats.images_processed),
                images_original_bytes: Some(image_stats.images_original_bytes),
                images_compressed_bytes: Some(image_stats.images_compressed_bytes),
                streams_compressed: Some(stream_stats.streams_compressed),
                fonts_deduplicated: None,
                metadata_removed_bytes: None,
            }),
        });
    }

    // Compressed file is smaller — move it into place
    std::fs::rename(&temp_path, &output_path)
        .with_context(|| format!("Failed to rename output: {}", output_path.display()))?;

    let pct = reduction_percent(original_size, temp_size);

    Ok(FileResult {
        input_path: input_path.display().to_string(),
        output_path: Some(output_path.display().to_string()),
        status: FileStatus::Compressed,
        original_size_bytes: original_size,
        output_size_bytes: Some(temp_size),
        reduction_percent: pct,
        pages: Some(page_count),
        skip_reason: None,
        signature_details: None,
        error: None,
        details: Some(CompressionDetails {
            images_processed: Some(image_stats.images_processed),
            images_original_bytes: Some(image_stats.images_original_bytes),
            images_compressed_bytes: Some(image_stats.images_compressed_bytes),
            streams_compressed: Some(stream_stats.streams_compressed),
            fonts_deduplicated: None,
            metadata_removed_bytes: None,
        }),
    })
}

/// Run the compression pipeline on all input files.
pub fn run_pipeline(
    inputs: &[PathBuf],
    config: &CompressionConfig,
    force: bool,
    preset_name: &str,
) -> Report {
    use std::time::Instant;

    let start = Instant::now();
    let mut files = Vec::new();
    let mut total_original = 0u64;
    let mut total_output = 0u64;
    let mut compressed_count = 0usize;
    let mut skipped_count = 0usize;
    let mut error_count = 0usize;

    for input in inputs {
        let result = process_file(input, config, force);
        match result {
            Ok(fr) => {
                match fr.status {
                    FileStatus::Compressed => compressed_count += 1,
                    FileStatus::Skipped => skipped_count += 1,
                    FileStatus::Error => error_count += 1,
                }
                total_original += fr.original_size_bytes;
                if let Some(out) = fr.output_size_bytes {
                    total_output += out;
                }
                files.push(fr);
            }
            Err(e) => {
                error_count += 1;
                files.push(FileResult {
                    input_path: input.display().to_string(),
                    output_path: None,
                    status: FileStatus::Error,
                    original_size_bytes: 0,
                    output_size_bytes: None,
                    reduction_percent: 0.0,
                    pages: None,
                    skip_reason: None,
                    signature_details: None,
                    error: Some(format!("Processing error: {}", e)),
                    details: None,
                });
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total_reduction = reduction_percent(total_original, total_output);

    Report {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        preset: preset_name.to_string(),
        summary: Summary {
            total_files: inputs.len(),
            compressed: compressed_count,
            skipped: skipped_count,
            errors: error_count,
            total_original_size_bytes: total_original,
            total_output_size_bytes: total_output,
            total_reduction_percent: total_reduction,
            elapsed_seconds: elapsed.round() as f64 / 100.0,
        },
        files,
    }
}
