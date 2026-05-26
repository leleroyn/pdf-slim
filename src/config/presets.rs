use crate::config::cli::Preset;

/// Compression configuration derived from preset + CLI overrides
#[derive(Clone, Debug)]
pub struct CompressionConfig {
    pub jpeg_quality: u8,
    pub max_dimension: u32,
    pub png_to_jpeg: bool,
    pub compress_streams: bool,
    pub remove_unused_fonts: bool,
}

impl Preset {
    pub fn to_config(&self) -> CompressionConfig {
        match self {
            Preset::Gentle => CompressionConfig {
                jpeg_quality: 85,
                max_dimension: 0,
                png_to_jpeg: false,
                compress_streams: true,
                remove_unused_fonts: false,
            },
            Preset::Balanced => CompressionConfig {
                jpeg_quality: 70,
                max_dimension: 1920,
                png_to_jpeg: true,
                compress_streams: true,
                remove_unused_fonts: false,
            },
            Preset::Aggressive => CompressionConfig {
                jpeg_quality: 50,
                max_dimension: 1280,
                png_to_jpeg: true,
                compress_streams: true,
                remove_unused_fonts: true,
            },
        }
    }
}

/// Apply CLI overrides on top of preset defaults
pub fn build_config(preset: Preset, image_quality: Option<u8>, max_dimension: Option<u32>) -> CompressionConfig {
    let mut config = preset.to_config();
    if let Some(q) = image_quality {
        config.jpeg_quality = q;
    }
    if let Some(d) = max_dimension {
        config.max_dimension = d;
    }
    config
}
