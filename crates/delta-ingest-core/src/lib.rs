//! Core telemetry model and traits used by Delta

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Game {
    F1_2024,
    F1_2025,
    GT7,
    LMU,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub game: Game,
    pub car_id: String,
    pub session_uid: String,
    pub frame: u64,
    pub sim_time_s: f64,

    // vehicle dynamics
    pub speed_mps: f32,
    pub throttle: f32,   // 0..1
    pub brake: f32,      // 0..1
    pub gear: i8,        // -1..8 etc.
    pub engine_rpm: f32,

    // world pose (right-handed, meters)
    pub world_pos_x: f32,
    pub world_pos_y: f32,
    pub world_pos_z: f32,
    pub yaw: f32,    // radians
    pub pitch: f32,  // radians
    pub roll: f32,   // radians

    // lap stuff
    pub lap_distance_m: f32,
    pub current_lap: u32,
    pub current_lap_time_s: f32,
    pub last_lap_time_s: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LapSummary {
    pub lap_number: u32,
    pub time_s: f32,
    pub sectors_s: Vec<f32>,
    pub best: bool,
    pub invalid: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("{0}")]
    Msg(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type TelemetryTx = crossbeam_channel::Sender<TelemetrySample>;
pub type TelemetryRx = crossbeam_channel::Receiver<TelemetrySample>;

/// Trait for any live source connector
#[async_trait::async_trait]
pub trait TelemetrySource: Send + Sync {
    async fn run(&self, tx: TelemetryTx) -> Result<(), IngestError>;
}

pub fn channel() -> (TelemetryTx, TelemetryRx) {
    crossbeam_channel::unbounded()
}
