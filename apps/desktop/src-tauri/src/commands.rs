// apps/desktop/src-tauri/src/commands.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct LapMetaInput {
    pub id: Uuid,
    pub game: String,
    pub track: String,
    pub car: String,
    pub lap_number: u32,
    pub time_ms: u64,
}

#[tauri::command]
pub async fn start_f1() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn start_gt7() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn start_lmu() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn stop_all() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn list_laps() -> Result<Vec<LapMetaInput>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn analyze_laps(_ids: Vec<Uuid>) -> Result<String, String> {
    Ok("analysis-ok".into())
}

#[tauri::command]
pub async fn build_track_map(_track: String) -> Result<String, String> {
    Ok("track-map-ok".into())
}

#[tauri::command]
pub async fn import_file(_path: String) -> Result<String, String> {
    Ok("import-ok".into())
}

#[tauri::command]
pub async fn export_file(_dest: String) -> Result<String, String> {
    Ok("export-ok".into())
}

#[tauri::command]
pub async fn cars_and_tracks() -> Result<(Vec<String>, Vec<String>), String> {
    Ok((Vec::new(), Vec::new()))
}

#[tauri::command]
pub async fn save_workspace(_name: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn load_workspace(_name: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn list_workspaces() -> Result<Vec<String>, String> {
    Ok(Vec::new())
}
