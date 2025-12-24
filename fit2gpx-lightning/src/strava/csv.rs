use anyhow::{Context, Result};
use csv::{Reader, StringRecord};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// A single activity record from Strava's activities.csv
#[derive(Debug, Clone)]
pub struct StravaCsvActivity {
    pub activity_id: String,
    pub activity_name: String,
    pub activity_type: String,
    pub activity_date: String,
    pub distance: String,
    pub filename: String,
    // Store all other CSV fields to preserve them if needed
    pub other_fields: HashMap<String, String>,
}

/// Handler for reading Strava's activities.csv file
pub struct CsvHandler {
    pub activities: Vec<StravaCsvActivity>,
    pub headers: StringRecord,
    // Track the column indices for known fields
    pub activity_id_idx: usize,
    pub activity_name_idx: usize,
    pub activity_type_idx: usize,
    pub activity_date_idx: usize,
    pub distance_idx: usize,
    pub filename_idx: usize,
}

impl CsvHandler {
    /// Extract and parse activities.csv from a Strava archive
    ///
    /// # Arguments
    /// * `archive_path` - Path to the Strava export ZIP file
    ///
    /// # Returns
    /// * `CsvHandler` instance with parsed activity records
    pub fn extract_from_archive(archive_path: &Path) -> Result<Self> {
        // Extract CSV from archive
        let file = File::open(archive_path).context(format!(
            "Failed to open archive: {}",
            archive_path.display()
        ))?;

        let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

        let mut csv_file = archive
            .by_name("activities.csv")
            .context("activities.csv not found in archive")?;

        let mut csv_content = String::new();
        csv_file
            .read_to_string(&mut csv_content)
            .context("Failed to read activities.csv")?;

        drop(csv_file);

        // Parse CSV
        Self::parse_csv_content(&csv_content)
    }

    /// Parse CSV from a standalone file
    ///
    /// # Arguments
    /// * `csv_path` - Path to the activities.csv file
    ///
    /// # Returns
    /// * `CsvHandler` instance with parsed activity records
    pub fn from_file(csv_path: &Path) -> Result<Self> {
        let csv_content = std::fs::read_to_string(csv_path)
            .context(format!("Failed to read CSV file: {}", csv_path.display()))?;

        Self::parse_csv_content(&csv_content)
    }

    /// Parse CSV content into structured records
    fn parse_csv_content(csv_content: &str) -> Result<Self> {
        let mut reader = Reader::from_reader(csv_content.as_bytes());

        let headers = reader
            .headers()
            .context("Failed to read CSV headers")?
            .clone();

        // Find column indices for known fields
        let activity_id_idx = headers
            .iter()
            .position(|h| h == "Activity ID")
            .context("Missing 'Activity ID' column")?;
        let activity_name_idx = headers
            .iter()
            .position(|h| h == "Activity Name")
            .context("Missing 'Activity Name' column")?;
        let activity_type_idx = headers
            .iter()
            .position(|h| h == "Activity Type")
            .context("Missing 'Activity Type' column")?;
        let activity_date_idx = headers
            .iter()
            .position(|h| h == "Activity Date")
            .context("Missing 'Activity Date' column")?;
        let distance_idx = headers
            .iter()
            .position(|h| h == "Distance")
            .context("Missing 'Distance' column")?;
        let filename_idx = headers
            .iter()
            .position(|h| h == "Filename")
            .context("Missing 'Filename' column")?;

        // Parse all records
        let mut activities = Vec::new();

        for result in reader.records() {
            let record = result.context("Failed to read CSV record")?;

            // Extract known fields
            let activity_id = record.get(activity_id_idx).unwrap_or("").to_string();
            let activity_name = record.get(activity_name_idx).unwrap_or("").to_string();
            let activity_type = record.get(activity_type_idx).unwrap_or("").to_string();
            let activity_date = record.get(activity_date_idx).unwrap_or("").to_string();
            let distance = record.get(distance_idx).unwrap_or("").to_string();
            let filename = record.get(filename_idx).unwrap_or("").to_string();

            // Store all other fields
            let mut other_fields = HashMap::new();
            for (i, value) in record.iter().enumerate() {
                if i != activity_id_idx
                    && i != activity_name_idx
                    && i != activity_type_idx
                    && i != activity_date_idx
                    && i != distance_idx
                    && i != filename_idx
                    && let Some(header) = headers.get(i)
                {
                    other_fields.insert(header.to_string(), value.to_string());
                }
            }

            activities.push(StravaCsvActivity {
                activity_id,
                activity_name,
                activity_type,
                activity_date,
                distance,
                filename,
                other_fields,
            });
        }

        Ok(Self {
            activities,
            headers,
            activity_id_idx,
            activity_name_idx,
            activity_type_idx,
            activity_date_idx,
            distance_idx,
            filename_idx,
        })
    }

    /// Find an activity by matching filename
    ///
    /// Matches if the CSV filename ends with the given filename
    /// (to handle cases where path prefixes differ)
    pub fn find_by_filename(&self, filename: &str) -> Option<&StravaCsvActivity> {
        self.activities
            .iter()
            .find(|a| a.filename.ends_with(filename))
    }

    /// Find an activity by activity ID
    pub fn find_by_id(&self, id: &str) -> Option<&StravaCsvActivity> {
        self.activities.iter().find(|a| {
            a.filename
                .split('/')
                .next_back()
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                == id
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parsing() {
        let csv_content = r#"Activity ID,Activity Date,Activity Name,Activity Type,Distance,Filename
1234567890,2024-01-01 10:00:00,Morning Run,Run,5000,activities/1234567890.fit.gz
1234567891,2024-01-02 15:30:00,Afternoon Ride,Ride,25000,activities/1234567891.fit.gz"#;

        let handler = CsvHandler::parse_csv_content(csv_content).unwrap();
        assert_eq!(handler.activities.len(), 2);
        assert_eq!(handler.activities[0].activity_id, "1234567890");
        assert_eq!(handler.activities[0].activity_name, "Morning Run");
        assert_eq!(handler.activities[1].activity_type, "Ride");
    }

    #[test]
    fn test_find_by_filename() {
        let csv_content = r#"Activity ID,Activity Date,Activity Name,Activity Type,Distance,Filename
1234567890,2024-01-01 10:00:00,Morning Run,Run,5000,activities/1234567890.fit.gz"#;

        let handler = CsvHandler::parse_csv_content(csv_content).unwrap();

        // Should find with just the filename
        let activity = handler.find_by_filename("1234567890.fit.gz");
        assert!(activity.is_some());
        assert_eq!(activity.unwrap().activity_name, "Morning Run");

        // Should not find non-existent file
        let not_found = handler.find_by_filename("9999999999.fit.gz");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_by_id() {
        let csv_content = r#"Activity ID,Activity Date,Activity Name,Activity Type,Distance,Filename
1234567890,2024-01-01 10:00:00,Morning Run,Run,5000,activities/1234567890.fit.gz"#;

        let handler = CsvHandler::parse_csv_content(csv_content).unwrap();

        let activity = handler.find_by_id("1234567890");
        assert!(activity.is_some());
        assert_eq!(activity.unwrap().activity_name, "Morning Run");
    }
}
