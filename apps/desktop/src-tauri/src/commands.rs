use serde::{Serialize, Deserialize};
use std::{collections::HashMap, fs, path::PathBuf};
use tauri::State;
use uuid::Uuid;

use model::*;
use analysis as an;
use iox as iox;
use crate::session::{AppSession, run_source};

#[derive(Serialize, Deserialize)]
struct LapRow {
    id: Uuid, game: String, track: String, car: String, lap_number: u32, time_ms: u64
}

#[tauri::command]
pub async fn start_f1(state: State<'_, AppSession>, port: u16, format: u16) -> Result<(), String> {
    use delta_ingest_f1::{F1Source, F1Config};
    let src = F1Source::new(F1Config { bind_addr: format!("0.0.0.0:{}", port), expected_format: format });
    let sess: &'static AppSession = unsafe { std::mem::transmute(state.inner() )};
    run_source(src, format!("f1:{}:{}", port, format), sess);
    Ok(())
}

#[tauri::command]
pub async fn start_gt7(state: State<'_, AppSession>, consoleIp: String, variant: String, bindPort: u16) -> Result<(), String> {
    use delta_ingest_gt7::{GT7Source, GT7Config};
    let cfg = GT7Config { bind_addr: format!("0.0.0.0:{}", bindPort), console_ip: consoleIp, packet_variant: variant.chars().next().unwrap_or('A') };
    let src = GT7Source::new(cfg);
    let sess: &'static AppSession = unsafe { std::mem::transmute(state.inner() )};
    run_source(src, "gt7".into(), sess);
    Ok(())
}

#[tauri::command]
pub async fn start_lmu(state: State<'_, AppSession>) -> Result<(), String> {
    #[cfg(windows)]
    {
        let src = delta_ingest_lmu::LMUSource::new();
        let sess: &'static AppSession = unsafe { std::mem::transmute(state.inner() )};
        run_source(src, "lmu".into(), sess);
        Ok(())
    }
    #[cfg(not(windows))]
    { Err("LMU shared memory is available on Windows only".into()) }
}

#[tauri::command]
pub async fn stop_all(_state: State<'_, AppSession>) -> Result<(), String> {
    // For simplicity, our sources end when their sockets close or process exits;
    // here we do nothing (stateless). In production, hold join handles & cancel tokens.
    Ok(())
}

#[tauri::command]
pub async fn list_laps(state: State<'_, AppSession>) -> Result<Vec<LapRow>, String> {
    let inner = state.inner.lock();
    let mut v: Vec<LapRow> = inner.laps.values().map(|l| LapRow {
        id: l.id, game: l.meta.game.clone(), track: l.meta.track.clone(), car: l.meta.car.clone(),
        lap_number: l.meta.lap_number, time_ms: l.total_time_ms
    }).collect();
    v.sort_by_key(|r| r.time_ms);
    Ok(v)
}

#[tauri::command]
pub async fn analyze_laps(state: State<'_, AppSession>, lapIds: Vec<Uuid>) -> Result<serde_json::Value, String> {
    let inner = state.inner.lock();
    let mut lap_refs: Vec<&Lap> = Vec::with_capacity(lapIds.len());
    for id in lapIds {
        if let Some(l) = inner.laps.get(&id) { 
            lap_refs.push(l); 
        }
    }
    if lap_refs.is_empty() { return Err("No laps".into()) }
    let ref_lap = lap_refs.iter().min_by_key(|l| l.total_time_ms).unwrap();
    let overlay = an::overlay_speed_vs_distance(&lap_refs);
    let delta_ribbon = an::rolling_delta_vs_reference(ref_lap, &lap_refs);
    let corners = an::per_corner_metrics(ref_lap);
    let summary = an::lap_summary(&lap_refs);
    Ok(serde_json::json!({
        "overlay": overlay,
        "delta_ribbon": delta_ribbon,
        "corners": corners,
        "summary": summary
    }))
}

#[tauri::command]
pub async fn build_track_map(state: State<'_, AppSession>, lapId: Uuid) -> Result<serde_json::Value, String> {
    let inner = state.inner.lock();
    let lap = inner.laps.get(&lapId).ok_or("Lap not found")?;
    let map = an::build_track_map(lap);
    Ok(serde_json::to_value(map).unwrap())
}

#[tauri::command]
pub async fn import_file(state: State<'_, AppSession>, path: String) -> Result<usize, String> {
    let mut inner = state.inner.lock();
    let p = PathBuf::from(path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let mut count = 0usize;
    match ext.as_str() {
        "csv" => {
            let laps = iox::import_csv(&p).map_err(|e| e.to_string())?;
            for lap in laps { inner.laps.insert(lap.id, lap); count+=1; }
        }
        "ndjson" | "jsonl" => {
            let laps = iox::import_ndjson(&p).map_err(|e| e.to_string())?;
            for lap in laps { inner.laps.insert(lap.id, lap); count+=1; }
        }
        _ => return Err("unsupported extension".into())
    }
    Ok(count)
}

#[tauri::command]
pub async fn export_file(state: State<'_, AppSession>, kind: String, path: String) -> Result<(), String> {
    let inner = state.inner.lock();
    let laps: Vec<&Lap> = inner.laps.values().collect();
    let p = PathBuf::from(path);
    match kind.as_str() {
        "csv" => iox::export_csv(&laps, &p).map_err(|e| e.to_string())?,
        "ndjson" => iox::export_ndjson(&laps, &p).map_err(|e| e.to_string())?,
        "motec_csv" => iox::export_motec_csv(&laps, &p).map_err(|e| e.to_string())?,
        _ => return Err("unsupported export kind".into())
    }
    Ok(())
}

#[tauri::command]
pub async fn cars_and_tracks(game: String) -> Result<serde_json::Value, String> {
    let data_dir = std::env::current_dir().unwrap().join("data").join("cars_tracks");
    let file = match game.as_str() {
        "f1_24" => data_dir.join("f1_24.json"),
        "f1_25" => data_dir.join("f1_25.json"),
        "lmu" => data_dir.join("lmu.json"),
        "gt7" => data_dir.join("gt7.json"),
        _ => return Err("unknown game".into())
    };
    let s = fs::read_to_string(file).map_err(|e| e.to_string())?;
    Ok(serde_json::from_str(&s).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn save_workspace(state: State<'_, AppSession>, name: String, payload: serde_json::Value) -> Result<(), String> {
    let mut inner = state.inner.lock();
    inner.workspaces.insert(name.clone(), payload.clone());
    let dir = workspace_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{}.json", name)), serde_json::to_vec_pretty(&payload).unwrap()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn load_workspace(state: State<'_, AppSession>, name: String) -> Result<serde_json::Value, String> {
    let inner = state.inner.lock();
    if let Some(v) = inner.workspaces.get(&name) { return Ok(v.clone()) }
    let p = workspace_dir().join(format!("{}.json", name));
    let s = fs::read_to_string(&p).map_err(|e| e.to_string())?;
    Ok(serde_json::from_str(&s).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn list_workspaces(_state: State<'_, AppSession>) -> Result<Vec<String>, String> {
    let dir = workspace_dir();
    let mut v = vec![];
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            if let Some(stem) = e.path().file_stem().and_then(|s| s.to_str()) {
                v.push(stem.to_string());
            }
        }
    }
    if !v.contains(&"Default".to_string()) { v.push("Default".into()); }
    Ok(v)
}

fn workspace_dir() -> std::path::PathBuf {
    let base = dirs_next::data_dir().unwrap_or(std::env::current_dir().unwrap());
    base.join("Delta").join("workspaces")
}
