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

    let object_ids = doc.traverse_objects(|_| {});

    for obj_id in object_ids {
        if let Ok(obj) = doc.get_object_mut(obj_id) {
            if let Ok(stream) = obj.as_stream_mut() {
                let before = stream.content.len();
                let Ok(()) = stream.compress() else { continue };
                let after = stream.content.len();
                if after < before {
                    stats.streams_compressed += 1;
                }
            }
        }
    }

    Ok(stats)
}
