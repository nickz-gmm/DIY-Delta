#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod session;
mod commands;

use commands::*;
use session::AppSession;
use tauri::Manager;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .manage(AppSession::new())
        .invoke_handler(tauri::generate_handler![
            start_f1, start_gt7, start_lmu, stop_all,
            list_laps, analyze_laps, build_track_map,
            import_file, export_file,
            cars_and_tracks,
            save_workspace, load_workspace, list_workspaces
        ])
        .run(tauri::generate_context!())
        .expect("error while running app");
}
