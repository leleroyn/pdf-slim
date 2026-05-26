use std::collections::HashSet;

use anyhow::{Context, Result};
use lopdf::Document;

use crate::utils::format_size;

/// Information gathered about a PDF for --info mode
#[derive(Clone, Debug)]
pub struct PdfInfo {
    pub path: String,
    pub size_bytes: u64,
    pub page_count: usize,
    pub images: Vec<ImageInfo>,
    pub fonts: Vec<FontInfo>,
    pub is_encrypted: bool,
    pub has_signature: bool,
}

#[derive(Clone, Debug)]
pub struct ImageInfo {
    pub page: usize,
    pub width: u32,
    pub height: u32,
    pub color_space: String,
    pub filters: String,
    pub size_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontInfo {
    pub name: String,
    pub font_type: String,
}

/// Analyze a PDF and return detailed information
pub fn analyze_pdf(path: &std::path::Path) -> Result<PdfInfo> {
    let size_bytes = std::fs::metadata(path)?.len();

    let doc = Document::load(path)
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("cross-reference") || msg.contains("cross reference") {
                anyhow::anyhow!(
                    "PDF has a damaged or non-standard cross-reference table: {}\n\
                     Try repairing with: qpdf --repair {} - > {}_fixed.pdf",
                    msg,
                    path.display(),
                    path.display()
                )
            } else {
                anyhow::anyhow!("Failed to load PDF: {}", msg)
            }
        })
        .with_context(|| format!("File: {}", path.display()))?;

    let is_encrypted = doc.is_encrypted();
    let has_signature = crate::analyze::check_digital_signature(&doc).is_some();
    let pages = doc.get_pages();
    let page_count = pages.len();

    let mut images = Vec::new();
    let mut fonts_set = HashSet::new();

    for (page_idx, page_id) in &pages {
        // Collect images
        if let Ok(page_images) = doc.get_page_images(*page_id) {
            for img in &page_images {
                images.push(ImageInfo {
                    page: *page_idx as usize,
                    width: img.width as u32,
                    height: img.height as u32,
                    color_space: img.color_space.clone().unwrap_or_default(),
                    filters: img.filters.as_ref().map(|f| f.join(", ")).unwrap_or_default(),
                    size_bytes: img.content.len(),
                });
            }
        }

        // Collect fonts
        if let Ok(page_fonts) = doc.get_page_fonts(*page_id) {
            for (font_name, font_dict) in &page_fonts {
                let name = String::from_utf8_lossy(font_name).to_string();
                let font_type = match font_dict.get(b"Subtype") {
                    Ok(obj) => match obj.as_name() {
                        Ok(name) => String::from_utf8_lossy(name).to_string(),
                        Err(_) => String::new(),
                    },
                    Err(_) => String::new(),
                };
                fonts_set.insert(FontInfo { name, font_type });
            }
        }
    }

    let mut fonts: Vec<FontInfo> = fonts_set.into_iter().collect();
    fonts.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(PdfInfo {
        path: path.display().to_string(),
        size_bytes,
        page_count,
        images,
        fonts,
        is_encrypted,
        has_signature,
    })
}

/// Print the analysis report to stdout
pub fn print_info(info: &PdfInfo) {
    println!("\n{}", "=".repeat(50));
    println!("  {}", info.path);
    println!("  Size: {}", format_size(info.size_bytes));
    println!("  Pages: {}", info.page_count);
    println!("  Encrypted: {}", info.is_encrypted);
    println!("  Digital Signature: {}", info.has_signature);
    println!();

    println!("  Images: {}", info.images.len());
    let total_image_bytes: usize = info.images.iter().map(|i| i.size_bytes).sum();
    println!("  Total image size: {}", format_size(total_image_bytes as u64));
    for img in &info.images {
        println!(
            "    Page {}  {}x{}  {}  {}  {}",
            img.page,
            img.width,
            img.height,
            img.color_space,
            img.filters,
            format_size(img.size_bytes as u64),
        );
    }
    println!();

    println!("  Fonts: {}", info.fonts.len());
    for font in &info.fonts {
        println!("    {} ({})", font.name, font.font_type);
    }
    println!("{}", "=".repeat(50));
}
