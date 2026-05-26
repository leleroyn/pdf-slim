mod analyze;
mod compress;
mod config;
mod utils;

use anyhow::Result;
use clap::Parser;

use crate::analyze::info::{analyze_pdf, print_info};
use crate::compress::run_pipeline;
use crate::config::cli::Cli;
use crate::config::presets::build_config;
use crate::utils::print_summary;

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Build config from preset + overrides
    let preset = cli.preset;
    let config = build_config(preset, cli.image_quality, cli.max_dimension);
    let preset_name = preset.name();

    // Info mode
    if cli.info {
        for input in &cli.input {
            let info = analyze_pdf(input)?;
            print_info(&info);
        }
        return Ok(());
    }

    // Compression pipeline
    let report = run_pipeline(&cli.input, &config, cli.force, preset_name);

    // Output report
    if cli.json {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
    } else {
        print_summary(&report);
    }

    // Write JSON file if requested
    if let Some(json_path) = &cli.json_file {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(json_path, json)?;
    }

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    if let Err(e) = run() {
        eprintln!("pdf-slim error: {}", e);
        std::process::exit(1);
    }
}
