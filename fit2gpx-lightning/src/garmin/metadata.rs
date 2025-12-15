use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

/// Garmin activity metadata from summarizedActivities.json
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GarminActivity {
    pub activity_id: Option<u64>,
    pub name: Option<String>,
    pub activity_type: Option<ActivityType>,
    pub start_time_gmt: Option<f64>,
    pub begin_timestamp: Option<f64>,
    pub distance: Option<f64>,
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

impl GarminActivity {
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
pub fn read_garmin_activities(archive_path: &Path) -> Result<Vec<GarminActivity>> {
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

/// Build an indexed map of activities by timestamp for fast lookup
///
/// Groups activities into 1-second buckets (timestamps rounded to nearest second)
///
/// # Arguments
/// * `activities` - Vector of Garmin activities
///
/// # Returns
/// * HashMap mapping timestamp buckets to activity indices
pub fn build_timestamp_index(activities: &[GarminActivity]) -> HashMap<i64, Vec<usize>> {
    let mut index: HashMap<i64, Vec<usize>> = HashMap::new();

    for (idx, activity) in activities.iter().enumerate() {
        if let Some(ts) = activity.timestamp() {
            // Convert milliseconds to seconds and round to nearest second
            let bucket = (ts / 1000.0).round() as i64;
            index.entry(bucket).or_insert_with(Vec::new).push(idx);
        }
    }

    index
}

/// Find an activity by matching FIT file timestamp
///
/// Uses a 10-second tolerance window to match FIT timestamps with JSON metadata
///
/// # Arguments
/// * `fit_ts` - Timestamp from FIT file (in seconds)
/// * `index` - Timestamp index from build_timestamp_index
/// * `activities` - Vector of Garmin activities
///
/// # Returns
/// * Reference to matching activity, or None if no match found
pub fn find_activity_by_timestamp<'a>(
    fit_ts: f64,
    index: &HashMap<i64, Vec<usize>>,
    activities: &'a [GarminActivity],
) -> Option<&'a GarminActivity> {
    let fit_ts_ms = fit_ts * 1000.0;
    let tolerance_ms = 10000.0; // 10 second tolerance

    let bucket = fit_ts.round() as i64;

    // Search nearby buckets (±10 seconds)
    for b in (bucket - 10)..=(bucket + 10) {
        if let Some(indices) = index.get(&b) {
            for &idx in indices {
                let activity = &activities[idx];
                if let Some(json_ts) = activity.timestamp()
                    && (json_ts - fit_ts_ms).abs() <= tolerance_ms
                {
                    return Some(activity);
                }
            }
        }
    }

    None
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

    #[test]
    fn test_timestamp_index() {
        let activities = vec![
            GarminActivity {
                activity_id: Some(1),
                name: Some("Activity 1".to_string()),
                activity_type: Some(ActivityType::String("Running".to_string())),
                start_time_gmt: Some(1609459200000.0), // Jan 1, 2021 00:00:00
                begin_timestamp: None,
                distance: Some(5000.0),
            },
            GarminActivity {
                activity_id: Some(2),
                name: Some("Activity 2".to_string()),
                activity_type: Some(ActivityType::String("Cycling".to_string())),
                start_time_gmt: Some(1609545600000.0), // Jan 2, 2021 00:00:00
                begin_timestamp: None,
                distance: Some(25000.0),
            },
        ];

        let index = build_timestamp_index(&activities);
        assert!(index.len() >= 2);
    }

    #[test]
    fn test_find_activity_by_timestamp() {
        let activities = vec![GarminActivity {
            activity_id: Some(1),
            name: Some("Activity 1".to_string()),
            activity_type: Some(ActivityType::String("Running".to_string())),
            start_time_gmt: Some(1609459200000.0), // Jan 1, 2021 00:00:00
            begin_timestamp: None,
            distance: Some(5000.0),
        }];

        let index = build_timestamp_index(&activities);

        // Exact match (timestamp in seconds)
        let fit_ts = 1609459200.0;
        let found = find_activity_by_timestamp(fit_ts, &index, &activities);
        assert!(found.is_some());
        assert_eq!(found.unwrap().activity_id, Some(1));

        // Within tolerance (5 seconds off)
        let fit_ts_close = 1609459205.0;
        let found_close = find_activity_by_timestamp(fit_ts_close, &index, &activities);
        assert!(found_close.is_some());

        // Outside tolerance (20 seconds off)
        let fit_ts_far = 1609459220.0;
        let not_found = find_activity_by_timestamp(fit_ts_far, &index, &activities);
        assert!(not_found.is_none());
    }
}
