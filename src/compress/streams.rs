use anyhow::Result;
use lopdf::Document;

/// Statistics for stream compression
#[derive(Clone, Debug, Default)]
pub struct StreamStats {
    pub streams_compressed: usize,
}

/// Scan all PDF streams and apply Flate compression to uncompressed ones.
pub fn compress_streams(doc: &mut Document) -> Result<StreamStats> {
    let mut stats = StreamStats::default();

    // Collect object IDs first, then compress
    let object_ids = doc.traverse_objects(|_| {});

    for obj_id in object_ids {
        if let Ok(obj) = doc.get_object_mut(obj_id) {
            if let Ok(stream) = obj.as_stream_mut() {
                let _before = stream.content.len();
                let _ = stream.compress();
                stats.streams_compressed += 1;
            }
        }
    }

    Ok(stats)
}
