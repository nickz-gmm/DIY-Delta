use tauri::State;
use uuid::Uuid;

use model::*;
use analysis as an;
use io as io;
use serde::{Serialize, Deserialize};
use std::{fs, path::PathBuf};
use serde_json::{json, Value};
use crate::session::AppSession;

#[derive(Debug, Deserialize, Serialize)]
#[tauri::command]
pub async fn save_lap(
    id: Uuid,
    game: String,
    track: String,
    car: String,
    lap_number: u32,
    time_ms: u64,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
    let src = GT7Source::new(cfg);
    run_source(src, "gt7".into(), sess);
    Ok(())

#[tauri::command]
    #[cfg(windows)]
        let src = delta_ingest_lmu::LMUSource::new();
        run_source(src, "lmu".into(), sess);
        Ok(())
    #[cfg(not(windows))]

#[tauri::command]
    // For simplicity, our sources end when their sockets close or process exits;
    // here we do nothing (stateless). In production, hold join handles & cancel tokens.
    Ok(())

#[tauri::command]
    let inner = state.inner.lock();
        id: l.id, game: l.meta.game.clone(), track: l.meta.track.clone(), car: l.meta.car.clone(),
        lap_number: l.meta.lap_number, time_ms: l.total_time_ms
    v.sort_by_key(|r| r.time_ms);
    Ok(v)

#[tauri::command]
    let inner = state.inner.lock();
    let mut laps = vec![];
    let ref_lap = laps.iter().min_by_key(|l| l.total_time_ms).unwrap().clone();
    let overlay = an::overlay_speed_vs_distance(&laps);
    let delta_ribbon = an::rolling_delta_vs_reference(&ref_lap, &laps);
    let corners = an::per_corner_metrics(&ref_lap);
    let summary = an::lap_summary(&laps);
        "overlay": overlay,
        "delta_ribbon": delta_ribbon,
        "corners": corners,
        "summary": summary

#[tauri::command]
    let inner = state.inner.lock();
    let lap = inner.laps.get(&lapId).ok_or("Lap not found")?.clone();
    let map = an::build_track_map(&lap);
    Ok(serde_json::to_value(map).unwrap())

#[tauri::command]
    let mut inner = state.inner.lock();
    let p = PathBuf::from(path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let mut count = 0usize;
            let laps = iox::import_csv(&p).map_err(|e| e.to_string())?;
            let laps = iox::import_ndjson(&p).map_err(|e| e.to_string())?;
        _ => return Err("unsupported extension".into())
    Ok(count)

#[tauri::command]
    let inner = state.inner.lock();
    let laps: Vec<Lap> = inner.laps.values().cloned().collect();
    let p = PathBuf::from(path);
        "csv" => iox::export_csv(&laps, &p).map_err(|e| e.to_string())?,
        "ndjson" => iox::export_ndjson(&laps, &p).map_err(|e| e.to_string())?,
        "motec_csv" => iox::export_motec_csv(&laps, &p).map_err(|e| e.to_string())?,
        _ => return Err("unsupported export kind".into())
    Ok(())

#[tauri::command]
    let data_dir = std::env::current_dir().unwrap().join("data").join("cars_tracks");
        "f1_24" => data_dir.join("f1_24.json"),
        "f1_25" => data_dir.join("f1_25.json"),
        "lmu" => data_dir.join("lmu.json"),
        "gt7" => data_dir.join("gt7.json"),
        _ => return Err("unknown game".into())
    let s = fs::read_to_string(file).map_err(|e| e.to_string())?;
    Ok(serde_json::from_str(&s).map_err(|e| e.to_string())?)

#[tauri::command]
    let mut inner = state.inner.lock();
    inner.workspaces.insert(name.clone(), payload.clone());
    let dir = workspace_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(())

#[tauri::command]
    let inner = state.inner.lock();
    let s = fs::read_to_string(&p).map_err(|e| e.to_string())?;
    Ok(serde_json::from_str(&s).map_err(|e| e.to_string())?)

#[tauri::command]
    let dir = workspace_dir();
    let mut v = vec![];
                v.push(stem.to_string());
    Ok(v)

    let base = dirs_next::data_dir().unwrap_or(std::env::current_dir().unwrap());
    base.join("Delta").join("workspaces")
