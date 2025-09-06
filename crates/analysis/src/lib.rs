use model::*;

pub fn overlay_speed_vs_distance(laps: &Vec<Lap>) -> serde_json::Value {
    let max_len = laps.iter().map(|l| l.points.last().map(|p| p.lap_distance_m).unwrap_or(0.0)).fold(0.0, f64::max);
    let step = 1.0;
    let mut rows = vec![];
    let mut d = 0.0;
    while d <= max_len {
        let mut row = serde_json::Map::new();
        row.insert("distance".into(), serde_json::json!(d));
        for lap in laps {
            let v = sample_speed_at_distance(lap, d);
            row.insert(format!("speed_{}", lap.id), serde_json::json!(v));
        }
        rows.push(serde_json::Value::Object(row));
        d += step;
    }
    serde_json::Value::Array(rows)
}

fn sample_speed_at_distance(lap: &Lap, dist: f64) -> f64 {
    let mut best = 0.0;
    let mut bd = f64::INFINITY;
    for p in &lap.points {
        let dd = (p.lap_distance_m - dist).abs();
        if dd < bd { bd = dd; best = p.speed_kph; }
    }
    best
}

pub fn lap_summary(laps: &Vec<Lap>) -> serde_json::Value {
    let best = laps.iter().map(|l| l.total_time_ms).min().unwrap_or(0);
    let worst = laps.iter().map(|l| l.total_time_ms).max().unwrap_or(0);
    let avg = if !laps.is_empty() { laps.iter().map(|l| l.total_time_ms as f64).sum::<f64>()/(laps.len() as f64) } else { 0.0 };
    let mut sector_means = vec![];
    for l in laps {
        let split = thirds(l);
        sector_means.extend(split.into_iter().map(|x| x as f64));
    }
    let consistency = stddev(&sector_means);
    serde_json::json!({
        "best_ms": best,
        "worst_ms": worst,
        "avg_ms": avg,
        "consistency": consistency
    })
}

fn thirds(l: &Lap) -> Vec<u64> {
    let n = l.points.len().max(1);
    let s = n/3;
    let mut v = vec![];
    for i in 0..3 {
        let a = i*s;
        let b = if i==2 { n-1 } else { ((i+1)*s).min(n-1) };
        let t = (l.points[b].t_ms - l.points[a].t_ms).max(0.0) as u64;
        v.push(t);
    }
    v
}

fn stddev(v:&Vec<f64>) -> f64 {
    if v.is_empty() { return 0.0 }
    let m = v.iter().sum::<f64>()/(v.len() as f64);
    let var = v.iter().map(|x| (x-m)*(x-m)).sum::<f64>()/(v.len() as f64);
    var.sqrt()/1000.0
}

pub fn rolling_delta_vs_reference(reference: &Lap, laps: &Vec<Lap>) -> serde_json::Value {
    let max_len = reference.points.last().map(|p| p.lap_distance_m).unwrap_or(0.0);
    let step = 1.0;
    let mut rows = vec![];
    let mut d = 0.0;
    while d <= max_len {
        let t_ref = time_at_distance(reference, d);
        let mut delta = 0.0;
        let mut count = 0.0;
        for lap in laps {
            if lap.id == reference.id { continue; }
            let t = time_at_distance(lap, d);
            delta += (t - t_ref);
            count += 1.0;
        }
        if count > 0.0 { delta /= count; }
        rows.push(serde_json::json!({
            "distance": d,
            "delta_ms": delta
        }));
        d += step;
    }
    serde_json::Value::Array(rows)
}

fn time_at_distance(lap: &Lap, dist: f64) -> f64 {
    let mut best_t = lap.points.last().map(|p| p.t_ms).unwrap_or(0.0);
    let mut bd = f64::INFINITY;
    for p in &lap.points {
        let dd = (p.lap_distance_m - dist).abs();
        if dd < bd { bd = dd; best_t = p.t_ms; }
    }
    best_t - lap.points.first().map(|p| p.t_ms).unwrap_or(0.0)
}

pub fn build_track_map(lap: &Lap) -> TrackMap {
    let pl: Vec<Point2> = lap.points.iter().map(|p| Point2{ x:p.x, y:p.y }).collect();
    let bbox = bbox_of(&pl);
    let curv = curvature_series(&lap.points);
    let peaks = peak_indices(&curv, 12, 0.03);
    let mut corners = vec![];
    for (i, idx) in peaks.iter().enumerate() {
        let p = &lap.points[*idx];
        corners.push(CornerLabel{ index: (i+1) as u32, x: p.x, y: p.y });
    }
    let sectors = auto_sectors(lap, &curv, 3);
    TrackMap { polyline: pl, corners, sectors, bbox }
}

fn bbox_of(pl: &Vec<Point2>) -> BBox {
    let (mut minx, mut maxx, mut miny, mut maxy) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for p in pl {
        if p.x<minx {minx=p.x} if p.x>maxx {maxx=p.x}
        if p.y<miny {miny=p.y} if p.y>maxy {maxy=p.y}
    }
    BBox{minx,maxx,miny,maxy}
}

fn curvature_series(points: &Vec<TelemetryPoint>) -> Vec<f64> {
    let n = points.len();
    let mut c = vec![0.0; n];
    for i in 1..n-1 {
        let p0 = &points[i-1]; let p1 = &points[i]; let p2 = &points[i+1];
        let dx1 = p1.x - p0.x; let dy1 = p1.y - p0.y;
        let dx2 = p2.x - p1.x; let dy2 = p2.y - p1.y;
        let cross = (dx1*dy2 - dy1*dx2).abs();
        let den = ((dx1*dx1 + dy1*dy1).sqrt() * (dx2*dx2 + dy2*dy2).sqrt() * ((dx1+dx2)*(dx1+dx2) + (dy1+dy2)*(dy1+dy2)).sqrt()).max(1e-6);
        c[i] = cross/den;
    }
    let mut s = vec![0.0; n];
    for i in 0..n {
        let mut sum = 0.0; let mut cnt = 0.0;
        for k in i.saturating_sub(2)..(i+3).min(n) { sum += c[k]; cnt += 1.0; }
        s[i] = sum/cnt;
    }
    s
}

fn peak_indices(curv: &Vec<f64>, window: usize, threshold: f64) -> Vec<usize> {
    let n = curv.len();
    let mut peaks = vec![];
    for i in window..(n-window) {
        let v = curv[i];
        if v < threshold { continue; }
        let mut is_peak = true;
        for k in i-window..i+window {
            if curv[k] > v { is_peak = false; break; }
        }
        if is_peak { peaks.push(i); }
    }
    peaks
}

fn auto_sectors(lap: &Lap, curv: &Vec<f64>, n: usize) -> Vec<Sector> {
    let mut idx: Vec<(usize,f64)> = (0..curv.len()).map(|i| (i, curv[i])).collect();
    idx.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());
    let mut cuts: Vec<usize> = idx.iter().take(n-1).map(|(i,_)| *i).collect();
    cuts.sort();
    let mut ds = vec![0.0];
    for c in cuts { ds.push(lap.points[c].lap_distance_m); }
    ds.push(lap.points.last().map(|p| p.lap_distance_m).unwrap_or(0.0));
    let mut sectors = vec![];
    for w in ds.windows(2) {
        sectors.push(Sector{ start_m: w[0], end_m: w[1] });
    }
    sectors
}

pub fn per_corner_metrics(reference: &Lap) -> Vec<serde_json::Value> {
    let curv = curvature_series(&reference.points);
    let peaks = peak_indices(&curv, 12, 0.03);
    let mut out = vec![];
    for (i, idx) in peaks.iter().enumerate() {
        let apex = &reference.points[*idx];
        let window = 20usize;
        let start = idx.saturating_sub(window);
        let end = (*idx+window).min(reference.points.len()-1);
        let entry = reference.points[start].speed_kph;
        let exit = reference.points[end].speed_kph;
        let min_speed = (start..=end).map(|k| reference.points[k].speed_kph).fold(f64::INFINITY, f64::min);
        let mut brake_m = apex.lap_distance_m;
        let mut throt_m = apex.lap_distance_m;
        for k in (start..*idx).rev() {
            if reference.points[k].brake > 0.2 { brake_m = reference.points[k].lap_distance_m; }
        }
        for k in *idx..=end {
            if reference.points[k].throttle > 0.6 { throt_m = reference.points[k].lap_distance_m; break; }
        }
        out.push(serde_json::json!({
            "index": i+1,
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
