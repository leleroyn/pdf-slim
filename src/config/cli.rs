use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// PDF Slim - A fast, lightweight PDF compression tool
#[derive(Parser, Debug)]
#[command(name = "pdf-slim", version, about, arg_required_else_help = true)]
pub struct Cli {
    /// Input PDF file(s) to process
    pub input: Vec<PathBuf>,

    /// Output file path (default: <name>.slim.pdf)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Compression preset
    #[arg(long, default_value = "balanced")]
    pub preset: Preset,

    /// JPEG quality (1-100, overrides preset)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub image_quality: Option<u8>,

    /// Maximum image dimension (0 = no downsampling)
    #[arg(long)]
    pub max_dimension: Option<u32>,

    /// Overwrite original file (with backup)
    #[arg(long)]
    pub force: bool,

    /// Analyze PDF without compressing
    #[arg(long)]
    pub info: bool,

    /// Output full report as JSON to stdout
    #[arg(long)]
    pub json: bool,

    /// Write JSON report to file
    #[arg(long)]
    pub json_file: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Preset {
    /// JPEG quality 85, no downsampling, preserve PNG
    Gentle,
    /// JPEG quality 70, max 1920px, PNG to JPEG (default)
    #[default]
    Balanced,
    /// JPEG quality 50, max 1280px, all PNG to JPEG
    Aggressive,
}

impl Preset {
    pub fn name(&self) -> &'static str {
        match self {
            Preset::Gentle => "gentle",
            Preset::Balanced => "balanced",
            Preset::Aggressive => "aggressive",
        }
    }
}
