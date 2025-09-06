use anyhow::Result;
use std::{fs::File, path::Path};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use model::*;

pub fn import_csv(path: &Path) -> Result<Vec<Lap>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut laps = Vec::<Lap>::new();
    let mut current: Option<Lap> = None;
    for rec in rdr.deserialize() {
        let r: CsvRow = rec?;
        if current.as_ref().map(|l| l.meta.lap_number) != Some(r.lap_number) {
            if let Some(l) = current.take() { laps.push(l); }
            current = Some(new_lap(&r));
        }
        if let Some(l) = &mut current {
            l.points.push(TelemetryPoint{
                t_ms: r.t_ms, lap_distance_m: r.lap_distance_m,
                x: r.x, y: r.y, speed_kph: r.speed_kph,
                throttle: r.throttle, brake: r.brake, gear: r.gear, rpm: r.rpm,
            });
            l.total_time_ms = r.t_ms as u64;
        }
    }
    if let Some(l) = current.take() { laps.push(l); }
    Ok(laps)
}

pub fn export_csv(laps: &Vec<Lap>, path: &Path) -> Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.serialize(CsvHeader::default())?;
    for l in laps {
        for p in &l.points {
            w.serialize(CsvRow{
                game: l.meta.game.clone(),
                car: l.meta.car.clone(),
                track: l.meta.track.clone(),
                lap_number: l.meta.lap_number,
                t_ms: p.t_ms,
                lap_distance_m: p.lap_distance_m,
                x: p.x, y: p.y,
                speed_kph: p.speed_kph,
                throttle: p.throttle, brake: p.brake, gear: p.gear, rpm: p.rpm,
            })?;
        }
    }
    w.flush()?;
    Ok(())
}

pub fn import_ndjson(path: &Path) -> Result<Vec<Lap>> {
    let f = File::open(path)?;
    let rdr = std::io::BufReader::new(f);
    let mut laps = vec![];
    for line in rdr.lines() {
        let s = line?;
        let l: Lap = serde_json::from_str(&s)?;
        laps.push(l);
    }
    Ok(laps)
}

pub fn export_ndjson(laps: &Vec<Lap>, path: &Path) -> Result<()> {
    let f = File::create(path)?;
    let mut w = std::io::BufWriter::new(f);
    for l in laps {
        let s = serde_json::to_string(l)?;
        use std::io::Write;
        writeln!(w, "{}", s)?;
    }
    w.flush()?;
    Ok(())
}

pub fn export_motec_csv(laps: &Vec<Lap>, path: &Path) -> Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record(&["Time","LapDistance","X","Y","Speed","Throttle","Brake","Gear","RPM","LapNumber","Track","Car","Game"])?;
    for l in laps {
        let t0 = l.points.first().map(|p| p.t_ms).unwrap_or(0.0);
        for p in &l.points {
            w.write_record(&[
                format!("{:.6}", (p.t_ms - t0)/1000.0),
                format!("{:.3}", p.lap_distance_m),
                format!("{:.4}", p.x),
                format!("{:.4}", p.y),
                format!("{:.3}", p.speed_kph),
                format!("{:.3}", p.throttle),
                format!("{:.3}", p.brake),
                format!("{}", p.gear),
                format!("{:.1}", p.rpm),
                format!("{}", l.meta.lap_number),
                l.meta.track.clone(),
                l.meta.car.clone(),
                l.meta.game.clone(),
            ])?;
        }
    }
    w.flush()?;
    Ok(())
}

fn new_lap(r: &CsvRow) -> Lap {
    Lap {
        id: Uuid::new_v4(),
        meta: LapMeta {
            id: Uuid::new_v4(),
            game: r.game.clone(),
            car: r.car.clone(),
            track: r.track.clone(),
            lap_number: r.lap_number,
        },
        total_time_ms: 0,
        points: vec![],
    }
}

#[derive(Serialize, Deserialize)]
struct CsvHeader {
    game: String, car: String, track: String, lap_number: u32,
    t_ms: f64, lap_distance_m: f64, x: f64, y: f64, speed_kph: f64,
    throttle: f64, brake: f64, gear: i8, rpm: f64,
}
impl Default for CsvHeader {
    fn default() -> Self {
        Self {
            game: "game".into(), car: "car".into(), track: "track".into(), lap_number: 0,
            t_ms: 0.0, lap_distance_m: 0.0, x: 0.0, y: 0.0, speed_kph: 0.0,
            throttle: 0.0, brake: 0.0, gear: 0, rpm: 0.0
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CsvRow {
    game: String, car: String, track: String, lap_number: u32,
    t_ms: f64, lap_distance_m: f64, x: f64, y: f64, speed_kph: f64,
    throttle: f64, brake: f64, gear: i8, rpm: f64,
}
