use std::fs::{self, File};
use std::io::Write;
use serde_json::Value;
use crate::config::{save_career_races, save_root, save_tt_races};
use crate::log;

pub fn save_race_info(mut race_info: Value) {
    
    if let Some(sim_data) = race_info.get("<SimDataBase64>k__BackingField") {
        if sim_data.is_null() {
            log!("[RaceInfo] Skipped saving: <SimDataBase64>k__BackingField is null.");
            return;
        }
    }

    if let Value::Object(ref mut map) = race_info {
        map.insert(
            "horseACT_version".to_string(), 
            Value::String(env!("CARGO_PKG_VERSION").to_string())
        );
    }

    let now = chrono::Local::now();
    let date_str = now.format("%Y%m%d").to_string();
    
    let mut filename = format!("{}.json", now.format("%Y%m%d_%H%M%S_%3f"));

    if let Some(horses) = race_info.get("<RaceHorse>k__BackingField").and_then(|v| v.as_array()) {
        let winner_opt = horses.iter().find(|h| {
            h.get("FinishOrder").and_then(|v| v.as_i64()) == Some(0)
        });

        if let Some(winner) = winner_opt {
            let name = winner
                .get("<charaName>k__BackingField")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            let raw_time = winner
                .get("FinishTimeRaw")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let safe_name: String = name
                .chars()
                .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
                .collect();

            filename = format!("{}-{:.4}s-{}.json", safe_name.trim(), raw_time, date_str);
        }
    }

    let folder = match race_info.get("<RaceType>k__BackingField").and_then(|v| v.as_str()) {
        Some("RoomMatch") => "Room match",
        Some("Champions") => "Champions meeting",
        Some("Single") => "Career",
        Some("Practice") => "Practice room",
        _ => "Other",
    };

    if folder == "Career" && !save_career_races() {
        log!("[RaceInfo] Skipped saving Career race because saveCareerRaces is disabled.");
        return;
    }

    let dir = save_root().join(folder);
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            log!("[RaceInfo] Failed to create dir {:?}: {}", dir, e);
            return;
        }
    }

    let path = dir.join(filename);

    match File::create(&path) {
        Ok(mut f) => {
            match serde_json::to_string_pretty(&race_info) {
                Ok(json_str) => {
                    if let Err(e) = write!(f, "{}", json_str) {
                        log!("[RaceInfo] Failed to write JSON: {}", e);
                    } else {
                        log!("[RaceInfo] Saved to: {}", path.display());
                    }
                }
                Err(e) => {
                    log!("[RaceInfo] Failed to serialize JSON: {}", e);
                }
            }
        }
        Err(e) => {
            log!("[RaceInfo] Failed to create file: {}", e);
        }
    }
}

pub fn save_team_trial_result(mut response: Value) {
    if !save_tt_races() {
        log!("[TeamTrials] Skipped saving because saveTTRaces is disabled.");
        return;
    }

    if let Value::Object(ref mut map) = response {
        map.insert(
            "horseACT_version".to_string(),
            Value::String(env!("CARGO_PKG_VERSION").to_string()),
        );
    }

    let now = chrono::Local::now();
    let filename = format!("TT-{}.json", now.format("%Y%m%d_%H%M%S_%3f"));
    let dir = save_root().join("Team trials");

    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            log!("[TeamTrials] Failed to create dir {:?}: {}", dir, e);
            return;
        }
    }

    let path = dir.join(filename);
    match File::create(&path) {
        Ok(mut f) => match serde_json::to_string_pretty(&response) {
            Ok(json_str) => {
                if let Err(e) = write!(f, "{}", json_str) {
                    log!("[TeamTrials] Failed to write JSON: {}", e);
                } else {
                    log!("[TeamTrials] Saved to: {}", path.display());
                }
            }
            Err(e) => {
                log!("[TeamTrials] Failed to serialize JSON: {}", e);
            }
        },
        Err(e) => {
            log!("[TeamTrials] Failed to create file: {}", e);
        }
    }
}

