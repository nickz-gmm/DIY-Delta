use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct LapMeta {
    #[serde(with = "uuid::serde::simple")]
    pub id: Uuid,
    pub game: String,
    pub car: String,
    pub track: String,
    pub lap_number: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Lap {
    #[serde(with = "uuid::serde::simple")]
    pub id: Uuid,
    pub meta: LapMeta,
    pub total_time_ms: u64,
    #[serde(default)]
    pub points: Vec<TelemetryPoint>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TrackMap {
    #[serde(default)]
    pub polyline: Vec<Point2>,
    #[serde(default)]
    pub corners: Vec<CornerLabel>,
    #[serde(default)]
    pub sectors: Vec<Sector>,
    pub bbox: BBox,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Sector {
    pub start_m: f64,
    pub end_m: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CornerLabel {
    pub index: u32,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct BBox {
    pub minx: f64,
    pub maxx: f64,
    pub miny: f64,
    pub maxy: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}
