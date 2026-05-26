use serde::Serialize;
use std::fmt;

/// Top-level JSON report
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub version: String,
    pub timestamp: String,
    pub preset: String,
    pub summary: Summary,
    pub files: Vec<FileResult>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Summary {
    pub total_files: usize,
    pub compressed: usize,
    pub skipped: usize,
    pub errors: usize,
    pub total_original_size_bytes: u64,
    pub total_output_size_bytes: u64,
    pub total_reduction_percent: f64,
    pub elapsed_seconds: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileResult {
    pub input_path: String,
    pub output_path: Option<String>,
    pub status: FileStatus,
    pub original_size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "is_zero")]
    pub reduction_percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<SkipReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_details: Option<SignatureDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<CompressionDetails>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Compressed,
    Skipped,
    Error,
}

impl fmt::Display for FileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileStatus::Compressed => write!(f, "compressed"),
            FileStatus::Skipped => write!(f, "skipped"),
            FileStatus::Error => write!(f, "error"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    DigitalSignatureDetected,
}

impl fmt::Display for SkipReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkipReason::DigitalSignatureDetected => write!(f, "digital signature detected"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SignatureDetails {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer: Option<String>,
    pub certificates_count: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CompressionDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images_processed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images_original_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images_compressed_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streams_compressed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fonts_deduplicated: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_removed_bytes: Option<u64>,
}

fn is_zero(value: &f64) -> bool {
    *value == 0.0
}
