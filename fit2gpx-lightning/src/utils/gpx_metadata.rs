use anyhow::{Context, Result};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;
use std::path::Path;

/// Add or update metadata (name and description) in a GPX file
///
/// This function modifies a GPX file by adding or updating the <name> and <desc>
/// tags within the first <trk> element. This is used to embed activity metadata
/// like the activity name and type into the GPX track.
///
/// # Arguments
/// * `gpx_path` - Path to the GPX file to modify
/// * `name` - Optional activity name to set in <name> tag
/// * `description` - Optional activity type/description to set in <desc> tag
///
/// # Returns
/// * `Ok(())` - If the GPX was successfully modified
/// * `Err` - If file I/O or XML parsing fails
pub fn add_metadata_to_gpx(
    gpx_path: &Path,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    // Read existing GPX file
    let content = std::fs::read_to_string(gpx_path)
        .context(format!("Failed to read GPX file: {}", gpx_path.display()))?;

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    let mut in_track = false;
    let mut metadata_added = false;
    let mut skip_until_end_tag: Option<Vec<u8>> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"trk" => {
                in_track = true;
                writer.write_event(Event::Start(e.clone()))?;

                // Add name and description after <trk> tag
                if !metadata_added {
                    if let Some(n) = name {
                        writer.write_event(Event::Start(BytesStart::new("name")))?;
                        writer.write_event(Event::Text(BytesText::new(n)))?;
                        writer.write_event(Event::End(BytesEnd::new("name")))?;
                    }

                    if let Some(d) = description {
                        writer.write_event(Event::Start(BytesStart::new("desc")))?;
                        writer.write_event(Event::Text(BytesText::new(d)))?;
                        writer.write_event(Event::End(BytesEnd::new("desc")))?;
                    }

                    metadata_added = true;
                }
            }
            Ok(Event::Start(ref e))
                if in_track && (e.name().as_ref() == b"name" || e.name().as_ref() == b"desc") =>
            {
                // Skip existing name/desc tags inside <trk> since we're replacing them
                skip_until_end_tag = Some(e.name().as_ref().to_vec());
                continue;
            }
            Ok(Event::End(ref e))
                if skip_until_end_tag.as_ref() == Some(&e.name().as_ref().to_vec()) =>
            {
                // We've reached the end of the tag we're skipping
                skip_until_end_tag = None;
                continue;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"trk" => {
                in_track = false;
                writer.write_event(Event::End(e.clone()))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                // Only write events if we're not skipping
                if skip_until_end_tag.is_none() {
                    writer.write_event(e)?;
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                ));
            }
        }
        buf.clear();
    }

    // Write back to file
    let result = writer.into_inner().into_inner();
    std::fs::write(gpx_path, result)
        .context(format!("Failed to write GPX file: {}", gpx_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_add_metadata_to_gpx() {
        let temp_dir = TempDir::new().unwrap();
        let gpx_path = temp_dir.path().join("test.gpx");

        // Create a simple GPX file
        let gpx_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="Test">
  <trk>
    <trkseg>
      <trkpt lat="40.0" lon="-105.0">
        <ele>1600</ele>
        <time>2024-01-01T12:00:00Z</time>
      </trkpt>
    </trkseg>
  </trk>
</gpx>"#;

        fs::write(&gpx_path, gpx_content).unwrap();

        // Add metadata
        add_metadata_to_gpx(&gpx_path, Some("Test Activity"), Some("Running")).unwrap();

        // Read back and verify
        let result = fs::read_to_string(&gpx_path).unwrap();
        assert!(result.contains("<name>Test Activity</name>"));
        assert!(result.contains("<desc>Running</desc>"));
    }

    #[test]
    fn test_update_existing_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let gpx_path = temp_dir.path().join("test.gpx");

        // Create a GPX file with existing metadata
        let gpx_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="Test">
  <trk>
    <name>Old Name</name>
    <desc>Old Type</desc>
    <trkseg>
      <trkpt lat="40.0" lon="-105.0">
        <ele>1600</ele>
      </trkpt>
    </trkseg>
  </trk>
</gpx>"#;

        fs::write(&gpx_path, gpx_content).unwrap();

        // Update metadata
        add_metadata_to_gpx(&gpx_path, Some("New Name"), Some("Cycling")).unwrap();

        // Read back and verify old values are replaced
        let result = fs::read_to_string(&gpx_path).unwrap();
        assert!(result.contains("<name>New Name</name>"));
        assert!(result.contains("<desc>Cycling</desc>"));
        assert!(!result.contains("Old Name"));
        assert!(!result.contains("Old Type"));
    }
}
