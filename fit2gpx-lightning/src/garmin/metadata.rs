use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

use crate::garmin::archive::FitFileInfo;

/// Garmin activity metadata from summarizedActivities.json
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GarminActivity<'a> {
    pub activity_id: Option<u64>,
    pub name: Option<String>,
    pub activity_type: Option<ActivityType>,
    pub start_time_gmt: Option<f64>,
    pub begin_timestamp: Option<f64>,
    pub distance: Option<f64>,
    #[serde(skip)]
    pub matched_file: Option<&'a FitFileInfo>,
}

/// Activity type can be either a string or an object with type_key
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ActivityType {
    String(String),
    Object { type_key: String },
}

impl ActivityType {
    /// Get the activity type as a string
    pub fn type_key(&self) -> String {
        match self {
            ActivityType::String(s) => s.clone(),
            ActivityType::Object { type_key } => type_key.clone(),
        }
    }
}

impl<'a> GarminActivity<'a> {
    /// Get the timestamp in seconds (tries start_time_gmt, then begin_timestamp)
    pub fn timestamp(&self) -> Option<f64> {
        self.start_time_gmt.or(self.begin_timestamp)
    }

    /// Get activity name, or default to "Unknown Activity"
    pub fn name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| "Unknown Activity".to_string())
    }

    /// Get activity type as string
    pub fn type_string(&self) -> String {
        self.activity_type
            .as_ref()
            .map(|t| t.type_key())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

/// Extract Garmin activities from summarizedActivities.json in the archive
///
/// # Arguments
/// * `archive_path` - Path to the Garmin export ZIP file
///
/// # Returns
/// * Vector of GarminActivity structures sorted by timestamp (ascending)
pub fn read_garmin_activities(archive_path: &Path) -> Result<Vec<GarminActivity<'static>>> {
    let file = File::open(archive_path).context(format!(
        "Failed to open archive: {}",
        archive_path.display()
    ))?;

    let mut archive =
        ZipArchive::new(BufReader::new(file)).context("Failed to read ZIP archive")?;

    let mut all_activities = Vec::new();

    // Scan all files in the archive for summarizedActivities.json
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.contains("summarizedActivities.json") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let json: serde_json::Value = serde_json::from_str(&contents)
                .context(format!("Failed to parse JSON from: {}", name))?;

            // Garmin exports can have nested structure: [[{"summarizedActivitiesExport": [...]}]]
            if let Some(array) = json.as_array()
                && let Some(first) = array.first()
                && let Some(export) = first.get("summarizedActivitiesExport")
                && let Some(activities) = export.as_array()
            {
                for activity in activities {
                    if let Ok(act) = serde_json::from_value::<GarminActivity>(activity.clone()) {
                        all_activities.push(act);
                    }
                }
            }
        }
    }

    // Sort activities by timestamp
    all_activities.sort_by(|a, b| {
        a.timestamp()
            .partial_cmp(&b.timestamp())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(all_activities)
}

/// Linear scan matching of activities to FIT files based on timestamps
///
/// First, order activities and FIT files by timestamp
/// Then, for each activity, find the closest FIT file within a 10-second tolerance. Discard unmatched FIT files.
///
/// Note: Activities use millisecond timestamps, FIT files use second timestamps
///
/// # Arguments
/// * `activities` - Slice of GarminActivity
/// * `fit_files` - Slice of FitFile
/// # Returns
/// * Vector of (activity_idx, fit_file_idx) pairs for matched items
pub fn linear_activity_matching(
    activities: &[GarminActivity],
    fit_files: &[FitFileInfo],
) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();

    // Create sorted list of (timestamp_ms, activity_idx) pairs
    let mut activity_list: Vec<(i64, usize)> = activities
        .iter()
        .enumerate()
        .filter_map(|(idx, activity)| activity.timestamp().map(|ts| (ts as i64, idx)))
        .collect();
    activity_list.sort_by_key(|(ts, _)| *ts);

    // Create sorted list of (timestamp_ms, fit_file_idx) pairs
    // Convert FIT file timestamps from seconds to milliseconds
    let mut file_list: Vec<(i64, usize)> = fit_files
        .iter()
        .enumerate()
        .filter_map(|(idx, fit_file)| fit_file.timestamp.map(|ts| ((ts * 1000.0) as i64, idx)))
        .collect();
    file_list.sort_by_key(|(ts, _)| *ts);

    let tolerance_ms = 10000_i64; // 10 second tolerance
    let mut file_idx = 0;

    for (activity_ts, activity_idx) in &activity_list {
        // Skip FIT files that are too early (more than 10 seconds before activity)
        while file_idx < file_list.len() && file_list[file_idx].0 < activity_ts - tolerance_ms {
            file_idx += 1;
        }

        // Try to find a match within tolerance
        let mut best_match: Option<(i64, usize)> = None;
        let mut best_distance = i64::MAX;

        for (fit_ts, fit_idx) in file_list.iter().skip(file_idx) {
            let (fit_ts, fit_idx) = (*fit_ts, *fit_idx);
            let distance = (fit_ts - activity_ts).abs();

            // Stop if we've gone beyond the tolerance window
            if fit_ts > activity_ts + tolerance_ms {
                break;
            }

            // Track the closest match
            if distance < best_distance {
                best_distance = distance;
                best_match = Some((fit_ts, fit_idx));
            }
        }

        if let Some((_, fit_idx)) = best_match {
            matches.push((*activity_idx, fit_idx));
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_type() {
        let type_string = ActivityType::String("Running".to_string());
        assert_eq!(type_string.type_key(), "Running");

        let type_object = ActivityType::Object {
            type_key: "Cycling".to_string(),
        };
        assert_eq!(type_object.type_key(), "Cycling");
    }

    fn create_test_activity(timestamp_ms: f64, id: u64) -> GarminActivity<'static> {
        GarminActivity {
            activity_id: Some(id),
            name: Some(format!("Activity {}", id)),
            activity_type: Some(ActivityType::String("Running".to_string())),
            start_time_gmt: Some(timestamp_ms),
            begin_timestamp: None,
            distance: Some(5000.0),
            matched_file: None,
        }
    }

    fn create_test_fit_file(timestamp_sec: f64, filename: String) -> FitFileInfo {
        FitFileInfo {
            filename,
            data: vec![],
            timestamp: Some(timestamp_sec),
        }
    }

    #[test]
    fn test_linear_matching_exact_match() {
        // Activity at 1000000000000 ms (Sep 9, 2001)
        let activities = vec![create_test_activity(1000000000000.0, 1)];

        // FIT file at 1000000000 seconds (same time)
        let fit_files = vec![create_test_fit_file(1000000000.0, "test.fit".to_string())];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(matches.len(), 1, "Should match exactly");
        assert_eq!(
            matches[0],
            (0, 0),
            "Should match first activity to first file"
        );
    }

    #[test]
    fn test_linear_matching_within_tolerance() {
        // Activity at 1000000000000 ms
        let activities = vec![create_test_activity(1000000000000.0, 1)];

        // FIT file 5 seconds later (within 10 second tolerance)
        let fit_files = vec![create_test_fit_file(1000000005.0, "test.fit".to_string())];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(matches.len(), 1, "Should match within tolerance");
        assert_eq!(matches[0], (0, 0));
    }

    #[test]
    fn test_linear_matching_outside_tolerance() {
        // Activity at 1000000000000 ms
        let activities = vec![create_test_activity(1000000000000.0, 1)];

        // FIT file 15 seconds later (outside 10 second tolerance)
        let fit_files = vec![create_test_fit_file(1000000015.0, "test.fit".to_string())];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(matches.len(), 0, "Should not match outside tolerance");
    }

    #[test]
    fn test_linear_matching_multiple_activities() {
        let activities = vec![
            create_test_activity(1000000000000.0, 1), // Jan 1, 2001 at specific time
            create_test_activity(1000000100000.0, 2), // ~100 seconds later
            create_test_activity(1000000200000.0, 3), // ~200 seconds later
        ];

        let fit_files = vec![
            create_test_fit_file(1000000000.0, "file1.fit".to_string()),
            create_test_fit_file(1000000100.0, "file2.fit".to_string()),
            create_test_fit_file(1000000200.0, "file3.fit".to_string()),
        ];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(matches.len(), 3, "Should match all 3 activities");
        assert_eq!(matches[0], (0, 0));
        assert_eq!(matches[1], (1, 1));
        assert_eq!(matches[2], (2, 2));
    }

    #[test]
    fn test_linear_matching_chooses_closest() {
        // Activity at 1000000000000 ms
        let activities = vec![create_test_activity(1000000000000.0, 1)];

        // Two FIT files within tolerance, one closer than the other
        let fit_files = vec![
            create_test_fit_file(1000000002.0, "file1.fit".to_string()), // 2 seconds away
            create_test_fit_file(1000000008.0, "file2.fit".to_string()), // 8 seconds away
        ];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(matches.len(), 1, "Should match to exactly one file");
        assert_eq!(matches[0], (0, 0), "Should match to closest file (file1)");
    }

    #[test]
    fn test_linear_matching_unordered_input() {
        // Activities out of order
        let activities = vec![
            create_test_activity(1000000200000.0, 3),
            create_test_activity(1000000000000.0, 1),
            create_test_activity(1000000100000.0, 2),
        ];

        // FIT files out of order
        let fit_files = vec![
            create_test_fit_file(1000000200.0, "file3.fit".to_string()),
            create_test_fit_file(1000000000.0, "file1.fit".to_string()),
            create_test_fit_file(1000000100.0, "file2.fit".to_string()),
        ];

        let matches = linear_activity_matching(&activities, &fit_files);

        // Should still match correctly based on timestamps, not array order
        assert_eq!(matches.len(), 3, "Should match all 3 activities");
        // Matches should be sorted by activity timestamp
        assert_eq!(matches[0].0, 1); // activity idx 1 (earliest timestamp)
        assert_eq!(matches[1].0, 2); // activity idx 2 (middle timestamp)
        assert_eq!(matches[2].0, 0); // activity idx 0 (latest timestamp)
    }

    #[test]
    fn test_linear_matching_more_files_than_activities() {
        let activities = vec![create_test_activity(1000000000000.0, 1)];

        let fit_files = vec![
            create_test_fit_file(999999999.0, "file0.fit".to_string()), // Too early
            create_test_fit_file(1000000000.0, "file1.fit".to_string()), // Match
            create_test_fit_file(1000000001.0, "file2.fit".to_string()), // Also within tolerance
            create_test_fit_file(1000000100.0, "file3.fit".to_string()), // No matching activity
        ];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(matches.len(), 1, "Should only match one activity");
        assert_eq!(matches[0].1, 1, "Should match to exact timestamp file");
    }

    #[test]
    fn test_linear_matching_more_activities_than_files() {
        let activities = vec![
            create_test_activity(1000000000000.0, 1),
            create_test_activity(1000000100000.0, 2),
            create_test_activity(1000000200000.0, 3),
        ];

        let fit_files = vec![
            create_test_fit_file(1000000000.0, "file1.fit".to_string()),
            // Missing file for activity 2
            create_test_fit_file(1000000200.0, "file3.fit".to_string()),
        ];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(matches.len(), 2, "Should match 2 out of 3 activities");
        assert_eq!(matches[0], (0, 0));
        assert_eq!(matches[1], (2, 1));
    }

    #[test]
    fn test_linear_matching_empty_inputs() {
        let activities: Vec<GarminActivity> = vec![];
        let fit_files: Vec<FitFileInfo> = vec![];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(matches.len(), 0, "Empty inputs should produce no matches");
    }

    #[test]
    fn test_linear_matching_none_timestamps() {
        // Activity with None timestamp
        let activities = vec![GarminActivity {
            activity_id: Some(1),
            name: Some("Activity 1".to_string()),
            activity_type: Some(ActivityType::String("Running".to_string())),
            start_time_gmt: None,
            begin_timestamp: None,
            distance: Some(5000.0),
            matched_file: None,
        }];

        // FIT file with None timestamp
        let fit_files = vec![FitFileInfo {
            filename: "test.fit".to_string(),
            data: vec![],
            timestamp: None,
        }];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(
            matches.len(),
            0,
            "Activities/files without timestamps should not match"
        );
    }

    #[test]
    fn test_linear_matching_negative_offset() {
        // Activity at 1000000000000 ms
        let activities = vec![create_test_activity(1000000000000.0, 1)];

        // FIT file 5 seconds earlier (within tolerance, negative offset)
        let fit_files = vec![create_test_fit_file(999999995.0, "test.fit".to_string())];

        let matches = linear_activity_matching(&activities, &fit_files);

        assert_eq!(
            matches.len(),
            1,
            "Should match with negative offset within tolerance"
        );
    }
}
