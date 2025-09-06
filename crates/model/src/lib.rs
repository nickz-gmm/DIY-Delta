use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TelemetryPoint {
    pub t_ms: f64,
    pub lap_distance_m: f64,
    pub x: f64,
    pub y: f64,
    pub speed_kph: f64,
    pub throttle: f64,
    pub brake: f64,
    pub gear: i8,
    pub rpm: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LapMeta {
    pub id: Uuid,
    pub game: String,
    pub car: String,
    pub track: String,
    pub lap_number: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Lap {
    pub id: Uuid,
    pub meta: LapMeta,
    pub total_time_ms: u64,
    pub points: Vec<TelemetryPoint>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Corner {
    pub index: u32,
    pub start_m: f64,
    pub apex_m: f64,
    pub end_m: f64,
    pub x: f64,
    pub y: f64,
    pub min_speed: f64,
    pub entry_speed: f64,
    pub exit_speed: f64,
    pub brake_point_m: f64,
    pub throttle_on_m: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TrackMap {
    pub polyline: Vec<Point2>,
    pub corners: Vec<CornerLabel>,
    pub sectors: Vec<Sector>,
    pub bbox: BBox,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Sector {
    pub start_m: f64,
    pub end_m: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CornerLabel {
    pub index: u32,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BBox { pub minx: f64, pub maxx: f64, pub miny: f64, pub maxy: f64 }

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Point2 { pub x: f64, pub y: f64 }
