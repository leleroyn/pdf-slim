use anyhow::Result;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer};
use lopdf::{Document, Object, Stream};

use crate::config::presets::CompressionConfig;

/// Statistics for image processing
#[derive(Clone, Debug, Default)]
pub struct ImageStats {
    pub images_processed: usize,
    pub images_original_bytes: u64,
    pub images_compressed_bytes: u64,
}

/// Info collected about an image before processing (to avoid borrow checker issues)
struct ImageToProcess {
    object_id: lopdf::ObjectId,
    width: u32,
    height: u32,
    components: usize,
    data: Vec<u8>,
    original_bytes: u64,
    /// Original stream dictionary entries to preserve (ColorSpace, BitsPerComponent, etc.)
    original_dict: Vec<(Vec<u8>, Object)>,
    /// Filter chain (e.g. ["FlateDecode", "DCTDecode"])
    filters: Option<Vec<String>>,
}

/// Process all images in the PDF, compressing according to config.
pub fn compress_images(doc: &mut Document, config: &CompressionConfig) -> Result<ImageStats> {
    let mut stats = ImageStats::default();

    if config.jpeg_quality == 0 {
        return Ok(stats);
    }

    let pages = doc.get_pages();

    // First pass: collect all image data (immutable borrow)
    let mut to_process = Vec::new();

    for (_page_index, page_id) in &pages {
        let Ok(page_images) = doc.get_page_images(*page_id) else { continue };

        for pdf_image in &page_images {
            let original_bytes = pdf_image.content.len() as u64;

            // Skip tiny images
            if original_bytes < 1024 {
                continue;
            }

            // Skip CMYK images
            if let Some(ref cs) = pdf_image.color_space {
                if cs.contains("CMYK") || cs.contains("ICCBased") {
                    tracing::debug!("Skipping CMYK/ICCBased image");
                    continue;
                }
            }

            // Skip JPEG2000
            if let Some(ref filters) = pdf_image.filters {
                if filters.iter().any(|f| f == "JPXDecode") {
                    tracing::debug!("Skipping JPEG2000 image");
                    continue;
                }
            }

            // Skip non-8-bit images
            if let Some(bpc) = pdf_image.bits_per_component {
                if bpc != 8 {
                    tracing::debug!("Skipping {}-bit image", bpc);
                    continue;
                }
            }

            let width = pdf_image.width as u32;
            let height = pdf_image.height as u32;
            let color_space_str = pdf_image.color_space.clone();
            let components = match color_space_str.as_deref() {
                Some("DeviceGray") => 1,
                Some("DeviceRGB") => 3,
                _ => 3,
            };

            // Collect original stream dictionary to preserve metadata (ColorSpace, BitsPerComponent, etc.)
            let original_dict = if let Ok(obj) = doc.get_object(pdf_image.id) {
                if let Ok(stream) = obj.as_stream() {
                    stream.dict.iter().map(|(k, v)| (k.to_vec(), v.clone())).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            to_process.push(ImageToProcess {
                object_id: pdf_image.id,
                width,
                height,
                components,
                data: pdf_image.content.to_vec(),
                original_bytes,
                original_dict,
                filters: pdf_image.filters.clone(),
            });
        }
    }

    // Second pass: process and replace (mutable borrow)
    for img_info in to_process {
        // Decode image from raw data
        let Some(dynamic_img) = decode_pdf_image(&img_info.data, img_info.width, img_info.height, img_info.components, img_info.filters.as_deref()) else {
            continue;
        };
        let mut img = dynamic_img;

        let (new_width, new_height) = if config.max_dimension > 0
            && (img_info.width > config.max_dimension || img_info.height > config.max_dimension)
        {
            let scale = if img_info.width > img_info.height {
                config.max_dimension as f64 / img_info.width as f64
            } else {
                config.max_dimension as f64 / img_info.height as f64
            };
            let new_w = (img_info.width as f64 * scale) as u32;
            let new_h = (img_info.height as f64 * scale) as u32;
            img = img.resize(new_w, new_h, FilterType::Triangle);
            tracing::debug!("Downscaled {}x{} -> {}x{}", img_info.width, img_info.height, new_w, new_h);
            (new_w, new_h)
        } else {
            (img_info.width, img_info.height)
        };

        // Re-encode as JPEG
        let jpeg_data = match encode_jpeg(&img, config.jpeg_quality) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to encode image: {}", e);
                continue;
            }
        };
        let compressed_bytes = jpeg_data.len() as u64;

        // If compression made it bigger, keep original
        if compressed_bytes >= img_info.original_bytes {
            continue;
        }

        // Replace the image stream in the PDF
        if let Err(e) = replace_image_stream(
            doc,
            img_info.object_id,
            &jpeg_data,
            new_width,
            new_height,
            &img_info.original_dict,
        ) {
            tracing::warn!("Failed to replace image: {}", e);
            continue;
        }

        stats.images_processed += 1;
        stats.images_original_bytes += img_info.original_bytes;
        stats.images_compressed_bytes += compressed_bytes;
    }

    Ok(stats)
}

/// Decode raw PDF image data into a DynamicImage.
/// Handles DCTDecode (JPEG), FlateDecode+DCTDecode (zlib-wrapped JPEG), and raw pixel data.
fn decode_pdf_image(
    data: &[u8],
    width: u32,
    height: u32,
    components: usize,
    filters: Option<&[String]>,
) -> Option<DynamicImage> {
    use std::io::Read;

    if width == 0 || height == 0 {
        return None;
    }

    // If FlateDecode is in the filter chain, decompress first
    let has_flate = filters.is_some_and(|f| f.iter().any(|fl| fl == "FlateDecode" || fl == "LZWDecode"));

    let decompressed: Option<Vec<u8>>;
    if has_flate {
        let mut decoder = flate2::read::ZlibDecoder::new(data);
        let mut buf = Vec::with_capacity(data.len());
        if decoder.read_to_end(&mut buf).is_ok() {
            decompressed = Some(buf);
        } else {
            decompressed = None;
        }
    } else {
        decompressed = None;
    }

    let pixel_data = decompressed.as_ref().map(|d| d.as_slice()).unwrap_or(data);

    let expected_len = width as usize * height as usize * components;

    // If data length matches expected raw pixel size, it's uncompressed raw data
    if pixel_data.len() == expected_len {
        return match components {
            1 => ImageBuffer::from_raw(width, height, pixel_data.to_vec())
                .map(|img| DynamicImage::ImageLuma8(img)),
            3 => ImageBuffer::from_raw(width, height, pixel_data.to_vec())
                .map(|img| DynamicImage::ImageRgb8(img)),
            4 => ImageBuffer::from_raw(width, height, pixel_data.to_vec())
                .map(|img| DynamicImage::ImageRgba8(img)),
            _ => None,
        };
    }

    // Otherwise try decoding as JPEG (DCTDecode images store JPEG data)
    image::load_from_memory(pixel_data).ok()
}

/// Encode an image as JPEG at the specified quality
fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buffer, quality);
    img.write_with_encoder(encoder)?;
    Ok(buffer)
}

/// Replace the image stream data in the PDF, preserving original dictionary metadata.
fn replace_image_stream(
    doc: &mut Document,
    object_id: lopdf::ObjectId,
    new_data: &[u8],
    width: u32,
    height: u32,
    original_entries: &[(Vec<u8>, Object)],
) -> Result<()> {
    let mut new_dict = lopdf::Dictionary::new();

    // Restore original entries
    for (key, value) in original_entries {
        new_dict.set(key.clone(), value.clone());
    }

    // Override fields that must change
    new_dict.set(b"Width", Object::Integer(width as i64));
    new_dict.set(b"Height", Object::Integer(height as i64));
    new_dict.set(b"Filter", Object::Name(b"DCTDecode".to_vec()));
    new_dict.set(b"Length", Object::Integer(new_data.len() as i64));

    let new_stream = Stream::new(new_dict, new_data.to_vec());
    doc.set_object(object_id, new_stream);

    Ok(())
}
