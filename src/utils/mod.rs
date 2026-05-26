/// Format bytes to human-readable size
pub fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Compute reduction percentage
pub fn reduction_percent(original: u64, result: u64) -> f64 {
    if original == 0 {
        return 0.0;
    }
    ((1.0 - result as f64 / original as f64) * 100.0).round() as f64 / 100.0
}

/// Generate the default output path (<name>.slim.pdf)
pub fn default_output_path(input: &std::path::Path) -> std::path::PathBuf {
    let mut name = input.file_name().unwrap().to_string_lossy().to_string();
    if let Some(stem) = name.strip_suffix(".pdf") {
        name = format!("{}.slim.pdf", stem);
    } else {
        name.push_str(".slim.pdf");
    }
    let mut path = input.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
    path.push(name);
    path
}

/// Generate backup path
pub fn backup_path(input: &std::path::Path) -> std::path::PathBuf {
    if let Some(stem) = input.file_stem() {
        let backup_name = format!("{}.pdf-slim-bak", stem.to_string_lossy());
        input.with_file_name(backup_name)
    } else {
        input.with_extension(".pdf-slim-bak")
    }
}

/// Print human-readable summary to terminal
pub fn print_summary(report: &super::analyze::report::Report) {
    println!("\n{}", "=".repeat(60));
    println!("  pdf-slim {} — {}", report.version, report.preset);
    println!("{}", "=".repeat(60));

    let s = &report.summary;
    println!(
        "  Files: {} total, {} compressed, {} skipped, {} error",
        s.total_files, s.compressed, s.skipped, s.errors
    );
    println!(
        "  Size:  {} → {} ({:.1}% reduction)",
        format_size(s.total_original_size_bytes),
        format_size(s.total_output_size_bytes),
        s.total_reduction_percent
    );
    println!("  Time: {:.2}s", s.elapsed_seconds);

    for file in &report.files {
        let status_icon = match file.status {
            super::analyze::report::FileStatus::Compressed => "✓",
            super::analyze::report::FileStatus::Skipped => "⚠",
            super::analyze::report::FileStatus::Error => "✗",
        };
        let detail = match &file.output_size_bytes {
            Some(out) => format!(
                " {} → {} ({:.1}%)",
                format_size(file.original_size_bytes),
                format_size(*out),
                file.reduction_percent
            ),
            None => format!(" {}", format_size(file.original_size_bytes)),
        };
        let reason = match &file.skip_reason {
            Some(r) => format!(" ({})", r),
            None => String::new(),
        };
        let error_msg = file.error.as_ref().map(|e| format!(" — {}", e)).unwrap_or_default();
        println!("  {} {}{}{}", status_icon, file.input_path, detail, reason);
        if !error_msg.is_empty() {
            println!("      {}", error_msg);
        }
    }
    println!("{}", "=".repeat(60));
}
