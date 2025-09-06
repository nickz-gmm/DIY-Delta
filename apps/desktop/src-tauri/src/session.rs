use std::{collections::HashMap, thread, time::Duration};
use parking_lot::Mutex;
use serde_json::json;
use uuid::Uuid;

use model::*;
use delta_ingest_core::{TelemetrySample, TelemetryRx, TelemetrySource, channel, Game as GameId};
use analysis as an;

pub struct AppSession {
    pub inner: Mutex<Inner>,
}

pub struct Inner {
    pub laps: HashMap<Uuid, Lap>,
    pub workspaces: HashMap<String, serde_json::Value>,
    pub running: bool,
    // builders per source/session
    pub builders: HashMap<String, LapBuilder>,
    // join handles (we only need to drop them when stopping; simplified)
}

impl AppSession {
    pub fn new() -> Self { Self { inner: Mutex::new(Inner {
        laps: HashMap::new(),
        workspaces: HashMap::new(),
        running: false,
        builders: HashMap::new(),
    }) } }
}

// Build laps out of telemetry samples
pub struct LapBuilder {
    pub current: Option<Lap>,
    pub last: Option<TelemetrySample>,
    pub start_pos: Option<(f32,f32)>,
    pub cum_dist: f64,
    pub last_t_ms: f64,
    pub track_guess_m: f64,
}

impl LapBuilder {
    pub fn new(game: &str, car: &str, track: &str) -> Self {
        Self { current: Some(new_lap(game, car, track, 1)), last: None, start_pos: None, cum_dist: 0.0, last_t_ms: 0.0, track_guess_m: 0.0 }
    }
}

fn new_lap(game: &str, car: &str, track: &str, num: u32) -> Lap {
    Lap {
        id: Uuid::new_v4(),
        meta: LapMeta { id: Uuid::new_v4(), game: game.into(), car: car.into(), track: track.into(), lap_number: num },
        total_time_ms: 0,
        points: vec![]
    }
}

impl Inner {
    pub fn feed_sample(&mut self, key: &str, s: &TelemetrySample) {
        let (game, car, track) = (format!("{:?}", s.game).to_lowercase(), "Unknown", "Unknown");
        let b = self.builders.entry(key.to_string()).or_insert_with(|| LapBuilder::new(&game, car, track));
        // initialise start pos
        let posx = s.world_pos_x; let posy = s.world_pos_z;
        if b.start_pos.is_none() && s.speed_mps > 0.1 { b.start_pos = Some((posx, posy)); }

        // compute time and distance
        let t_ms = s.sim_time_s * 1000.0;
        let mut lap_dist = s.lap_distance_m as f64;
        if lap_dist <= 0.0 {
            if let Some(last) = &b.last {
                let dx = (s.world_pos_x - last.world_pos_x) as f64;
                let dy = (s.world_pos_z - last.world_pos_z) as f64;
                let step = (dx*dx + dy*dy).sqrt();
                b.cum_dist += step;
            }
            lap_dist = b.cum_dist;
        } else {
            b.cum_dist = lap_dist;
        }

        if let Some(lap) = &mut b.current {
            lap.points.push(TelemetryPoint {
                t_ms, lap_distance_m: lap_dist,
                x: posx as f64, y: posy as f64,
                speed_kph: (s.speed_mps * 3.6) as f64,
                throttle: s.throttle as f64,
                brake: s.brake as f64,
                gear: s.gear,
                rpm: s.engine_rpm as f64,
            });
            lap.total_time_ms = (t_ms - lap.points.first().map(|p| p.t_ms).unwrap_or(t_ms)) as u64;
        }

        // detect lap end
        let mut roll = false;
        // 1) explicit lap number increase
        if let Some(last) = &b.last {
            if s.current_lap > last.current_lap && s.current_lap > 0 {
                roll = true;
            }
        }
        // 2) heuristics when no lap numbers: near start pos and elapsed > 15s
        if !roll {
            if let (Some(sp), Some(lap)) = (b.start_pos, &b.current) {
                let dx = (posx - sp.0) as f64; let dy = (posy - sp.1) as f64;
                let d = (dx*dx + dy*dy).sqrt();
                let elapsed = t_ms - lap.points.first().map(|p| p.t_ms).unwrap_or(t_ms);
                if d < 20.0 && elapsed > 15000.0 && s.speed_mps > 1.0 { roll = true; }
            }
        }

        if roll {
            if let Some(mut finished) = b.current.take() {
                // sanity: set total time precisely
                finished.total_time_ms = (t_ms - finished.points.first().map(|p| p.t_ms).unwrap_or(t_ms)) as u64;
                // normalize lap distance to end value
                let lastd = finished.points.last().map(|p| p.lap_distance_m).unwrap_or(0.0);
                if lastd > b.track_guess_m { b.track_guess_m = lastd; }
                // insert
                self.laps.insert(finished.id, finished);
                // new lap
                let next_num = s.current_lap.max(1);
                b.current = Some(new_lap(&game, car, track, next_num));
                b.cum_dist = 0.0;
            }
        }

        b.last = Some(s.clone());
        b.last_t_ms = t_ms;
    }
}

pub fn run_source<S: TelemetrySource + 'static>(src: S, rx_key: String, sess: &'static AppSession) {
    let (tx, rx): (_, TelemetryRx) = channel();
    tokio::spawn(async move {
        let _ = src.run(tx).await;
    });
    // pump samples into session (blocking thread)
    std::thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(sample) => {
                    let mut inner = sess.inner.lock();
                    inner.feed_sample(&rx_key, &sample);
                }
                Err(_) => { thread::sleep(Duration::from_millis(10)); }
            }
        }
    });
}
