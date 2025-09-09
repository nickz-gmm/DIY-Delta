use model::*;
use serde_json::{json, Value};

pub fn overlay_speed_vs_distance(laps: &[Lap]) -> Value {
    let max_len = laps
        .iter()
        .filter_map(|l| l.points.last().map(|p| p.lap_distance_m))
        .fold(0.0_f64, f64::max);

    let step = 1.0_f64;
    let expected_rows = ((max_len / step) as usize).saturating_add(1);
    let mut rows = Vec::with_capacity(expected_rows);
    let mut d = 0.0_f64;

    while d <= max_len {
        let mut row = serde_json::Map::new();
        row.insert("distance".into(), json!(d));
        for lap in laps {
            let v = sample_speed_at_distance(lap, d);
            row.insert(format!("speed_{}", lap.id), json!(v));
        }
        rows.push(Value::Object(row));
        d += step;
    }

    Value::Array(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_lap() -> Lap {
        Lap {
            id: Uuid::new_v4(),
            meta: LapMeta {
                id: Uuid::new_v4(),
                game: "test".to_string(),
                car: "test_car".to_string(),
                track: "test_track".to_string(),
                lap_number: 1,
            },
            total_time_ms: 60000,
            points: vec![
                TelemetryPoint {
                    t_ms: 0.0,
                    lap_distance_m: 0.0,
                    x: 0.0,
                    y: 0.0,
                    speed_kph: 100.0,
                    throttle: 1.0,
                    brake: 0.0,
                    gear: 3,
                    rpm: 5000.0,
                },
                TelemetryPoint {
                    t_ms: 1000.0,
                    lap_distance_m: 50.0,
                    x: 10.0,
                    y: 5.0,
                    speed_kph: 120.0,
                    throttle: 0.8,
                    brake: 0.0,
                    gear: 4,
                    rpm: 5500.0,
                },
            ],
        }
    }

    #[test]
    fn test_lap_summary_with_references() {
        let lap1 = create_test_lap();
        let lap2 = create_test_lap();
        let laps = vec![lap1, lap2];
        
        let summary = lap_summary(&laps);
        assert!(summary["best_ms"].is_number());
        assert!(summary["worst_ms"].is_number());
        assert!(summary["avg_ms"].is_number());
        assert!(summary["consistency"].is_number());
    }

    #[test]
    fn test_overlay_speed_vs_distance() {
        let lap = create_test_lap();
        let laps = vec![lap];
        
        let overlay = overlay_speed_vs_distance(&laps);
        assert!(overlay.is_array());
        if let Value::Array(rows) = overlay {
            assert!(!rows.is_empty());
        }
    }

    #[test]
    fn test_build_track_map() {
        let lap = create_test_lap();
        let track_map = build_track_map(&lap);
        
        assert_eq!(track_map.polyline.len(), lap.points.len());
        assert!(!track_map.sectors.is_empty());
    }
}

fn sample_speed_at_distance(lap: &Lap, dist: f64) -> f64 {
    if lap.points.is_empty() {
        return 0.0;
    }
    let mut best = lap.points[0].speed_kph;
    let mut bd = f64::INFINITY;
    for p in &lap.points {
        let dd = (p.lap_distance_m - dist).abs();
        if dd < bd {
            bd = dd;
            best = p.speed_kph;
        }
    }
    best
}

pub fn lap_summary(laps: &[Lap]) -> Value {
    let best = laps.iter().map(|l| l.total_time_ms).min().unwrap_or(0);
    let worst = laps.iter().map(|l| l.total_time_ms).max().unwrap_or(0);
    let avg = if !laps.is_empty() {
        laps.iter().map(|l| l.total_time_ms as f64).sum::<f64>() / (laps.len() as f64)
    } else {
        0.0
    };

    // collect simple 3-way split sector times (ms) across all laps
    let mut sector_times_ms = Vec::with_capacity(laps.len() * 3);
    for l in laps {
        sector_times_ms.extend(thirds(l).into_iter().map(|x| x as f64));
    }
    let consistency = stddev(&sector_times_ms);

    json!({
        "best_ms": best,
        "worst_ms": worst,
        "avg_ms": avg,
        "consistency": consistency
    })
}

/// Very simple "thirds" segmentation over telemetry points.
/// Returns three elapsed-time segments (in ms) covering the lap.
fn thirds(l: &Lap) -> Vec<u64> {
    let n = l.points.len().max(1);
    let s = n / 3; // may be 0 for tiny laps; guarded by min/max below
    let mut v = Vec::with_capacity(3);

    for i in 0..3 {
        let a = i * s;
        let b = if i == 2 {
            n.saturating_sub(1)
        } else {
            ((i + 1) * s).min(n.saturating_sub(1))
        };
        let ta = l.points.get(a).map(|p| p.t_ms).unwrap_or(0.0);
        let tb = l.points.get(b).map(|p| p.t_ms).unwrap_or(ta);
        let t = (tb - ta).max(0.0) as u64;
        v.push(t);
    }

    v
}

fn stddev(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let m = v.iter().sum::<f64>() / (v.len() as f64);
    let var = v.iter().map(|x| {
        let d = *x - m;
        d * d
    }).sum::<f64>() / (v.len() as f64);
    // return seconds (input was ms)
    (var.sqrt()) / 1000.0
}

pub fn rolling_delta_vs_reference(reference: &Lap, laps: &[Lap]) -> Value {
    let max_len = reference
        .points
        .last()
        .map(|p| p.lap_distance_m)
        .unwrap_or(0.0);

    let step = 1.0_f64;
    let expected_rows = ((max_len / step) as usize).saturating_add(1);
    let mut rows = Vec::with_capacity(expected_rows);
    let mut d = 0.0_f64;

    while d <= max_len {
        let t_ref = time_at_distance(reference, d);
        let mut delta = 0.0_f64;
        let mut count = 0.0_f64;

        for lap in laps {
            if lap.id == reference.id {
                continue;
            }
            let t = time_at_distance(lap, d);
            delta += t - t_ref;
            count += 1.0;
        }

        if count > 0.0 {
            delta /= count;
        }

        rows.push(json!({
            "distance": d,
            "delta_ms": delta
        }));
        d += step;
    }

    Value::Array(rows)
}

fn time_at_distance(lap: &Lap, dist: f64) -> f64 {
    if lap.points.is_empty() {
        return 0.0;
    }

    let mut best_t = lap.points.last().map(|p| p.t_ms).unwrap_or(0.0);
    let mut bd = f64::INFINITY;

    for p in &lap.points {
        let dd = (p.lap_distance_m - dist).abs();
        if dd < bd {
            bd = dd;
            best_t = p.t_ms;
        }
    }

    let t0 = lap.points.first().map(|p| p.t_ms).unwrap_or(0.0);
    best_t - t0
}

pub fn build_track_map(lap: &Lap) -> TrackMap {
    let mut pl = Vec::with_capacity(lap.points.len());
    for p in &lap.points {
        pl.push(Point2 { x: p.x, y: p.y });
    }
    let bbox = bbox_of(&pl);
    let curv = curvature_series(&lap.points);
    let peaks = peak_indices(&curv, 12, 0.03);

    let mut corners = Vec::with_capacity(peaks.len());
    for (i, idx) in peaks.iter().enumerate() {
        if let Some(p) = lap.points.get(*idx) {
            corners.push(CornerLabel {
                index: (i + 1) as u32,
                x: p.x,
                y: p.y,
            });
        }
    }

    let sectors = auto_sectors(lap, &curv, 3);
    TrackMap { polyline: pl, corners, sectors, bbox }
}

fn bbox_of(pl: &[Point2]) -> BBox {
    let (mut minx, mut maxx, mut miny, mut maxy) =
        (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);

    for p in pl {
        if p.x < minx { minx = p.x; }
        if p.x > maxx { maxx = p.x; }
        if p.y < miny { miny = p.y; }
        if p.y > maxy { maxy = p.y; }
    }

    BBox { minx, maxx, miny, maxy }
}

fn curvature_series(points: &[TelemetryPoint]) -> Vec<f64> {
    let n = points.len();
    if n == 0 {
        return Vec::new();
    }

    let mut c = vec![0.0; n];
    // central-difference curvature proxy
    for i in 1..n.saturating_sub(1) {
        let p0 = &points[i - 1];
        let p1 = &points[i];
        let p2 = &points[i + 1];

        let dx1 = p1.x - p0.x;
        let dy1 = p1.y - p0.y;
        let dx2 = p2.x - p1.x;
        let dy2 = p2.y - p1.y;

        let cross = (dx1 * dy2 - dy1 * dx2).abs();

        let a = (dx1 * dx1 + dy1 * dy1).sqrt();
        let b = (dx2 * dx2 + dy2 * dy2).sqrt();
        let csum = ((dx1 + dx2) * (dx1 + dx2) + (dy1 + dy2) * (dy1 + dy2)).sqrt();

        let den = (a * b * csum).max(1e-6);
        c[i] = cross / den;
    }

    // light smoothing
    let mut s = vec![0.0; n];
    for i in 0..n {
        let mut sum = 0.0;
        let mut cnt = 0.0;
        let from = i.saturating_sub(2);
        let to = (i + 3).min(n);
        for k in from..to {
            sum += c[k];
            cnt += 1.0;
        }
        s[i] = if cnt > 0.0 { sum / cnt } else { 0.0 };
    }

    s
}

fn peak_indices(curv: &[f64], window: usize, threshold: f64) -> Vec<usize> {
    let n = curv.len();
    if n == 0 || window == 0 || n <= window * 2 {
        return Vec::new();
    }

    let mut peaks = Vec::new();
    for i in window..(n - window) {
        let v = curv[i];
        if v < threshold {
            continue;
        }
        // v must be a local maximum in the window
        let mut is_peak = true;
        for k in (i - window)..(i + window) {
            if curv[k] > v {
                is_peak = false;
                break;
            }
        }
        if is_peak {
            peaks.push(i);
        }
    }
    peaks
}

fn auto_sectors(lap: &Lap, curv: &[f64], n: usize) -> Vec<Sector> {
    if lap.points.is_empty() || n == 0 {
        return Vec::new();
    }

    // Choose the top (n-1) curvature peaks as cut points.
    let mut idx = Vec::with_capacity(curv.len());
    for (i, &v) in curv.iter().enumerate() {
        idx.push((i, v));
    }
    idx.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut cuts: Vec<usize> = idx.into_iter().take(n.saturating_sub(1)).map(|(i, _)| i).collect();
    cuts.sort_unstable();
    cuts.dedup();

    let mut ds = Vec::with_capacity(cuts.len() + 2);
    ds.push(lap.points.first().map(|p| p.lap_distance_m).unwrap_or(0.0));
    for c in cuts {
        if let Some(p) = lap.points.get(c) {
            ds.push(p.lap_distance_m);
        }
    }
    ds.push(lap.points.last().map(|p| p.lap_distance_m).unwrap_or(0.0));

    let mut sectors = Vec::with_capacity(ds.len().saturating_sub(1));
    for w in ds.windows(2) {
        sectors.push(Sector { start_m: w[0], end_m: w[1] });
    }
    sectors
}

pub fn per_corner_metrics(reference: &Lap) -> Vec<Value> {
    let curv = curvature_series(&reference.points);
    let peaks = peak_indices(&curv, 12, 0.03);
    let mut out = Vec::with_capacity(peaks.len());

    for (i, idx) in peaks.iter().enumerate() {
        if reference.points.is_empty() {
            break;
        }

        let apex = match reference.points.get(*idx) {
            Some(p) => p,
            None => continue,
        };

        // +/- window around the apex (clamped to valid indices)
        let window = 20usize;
        let start = idx.saturating_sub(window);
        let end = (*idx + window).min(reference.points.len().saturating_sub(1));

        let entry = reference.points[start].speed_kph;
        let exit = reference.points[end].speed_kph;
        let min_speed = (start..=end)
            .map(|k| reference.points[k].speed_kph)
            .fold(f64::INFINITY, f64::min);

        // first brake point before apex over threshold
        let mut brake_m = apex.lap_distance_m;
        for k in (start..*idx).rev() {
            if reference.points[k].brake > 0.2 {
                brake_m = reference.points[k].lap_distance_m;
            }
        }

        // first throttle-on after apex over threshold
        let mut throt_m = apex.lap_distance_m;
        for k in *idx..=end {
            if reference.points[k].throttle > 0.6 {
                throt_m = reference.points[k].lap_distance_m;
                break;
            }
        }

        out.push(json!({
            "index": i + 1,
            "start_m": reference.points[start].lap_distance_m,
            "apex_m": apex.lap_distance_m,
            "end_m": reference.points[end].lap_distance_m,
            "x": apex.x, "y": apex.y,
            "min_speed": min_speed,
            "entry_speed": entry,
            "exit_speed": exit,
            "brake_point_m": brake_m,
            "throttle_on_m": throt_m
        }));
    }

    out
}
