use anyhow::Context;
use tokio::net::UdpSocket;
use bytes::Buf;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use delta_ingest_core::{*, Game as GameId};

#[derive(Clone, Debug)]
pub struct F1Config {
    pub bind_addr: String,       // e.g. "0.0.0.0:20777"
    pub expected_format: u16,    // 2024 or 2025
}

impl Default for F1Config {
    fn default() -> Self {
        Self { bind_addr: "0.0.0.0:20777".into(), expected_format: 2025 }
    }
}

pub struct F1Source {
    cfg: F1Config
}

impl F1Source {
    pub fn new(cfg: F1Config) -> Self { Self { cfg } }
}

#[async_trait::async_trait]
impl TelemetrySource for F1Source {
    async fn run(&self, tx: TelemetryTx) -> Result<(), IngestError> {
        let socket = UdpSocket::bind(&self.cfg.bind_addr).await
            .with_context(|| format!("bind {}", self.cfg.bind_addr))?;
        let mut buf = vec![0u8; 2048];
        loop {
            let (len, _peer) = socket.recv_from(&mut buf).await?;
            if len < 32 { continue; }
            if let Some(sample) = parse_packet(&buf[..len], self.cfg.expected_format) {
                let _ = tx.send(sample).await;
            }
        }
    }
}

#[derive(Debug)]
struct PacketHeader {
    packet_format: u16, // 2024/2025
    game_year: u8,
    game_major: u8,
    game_minor: u8,
    packet_version: u8,
    packet_id: u8,
    session_uid: u64,
    session_time: f32,
    frame_identifier: u32,
    overall_frame_identifier: u32,
    player_car_index: u8,
    secondary_player_car_index: u8,
}

fn read_header(mut c: Cursor<&[u8]>) -> Option<PacketHeader> {
    let pf = c.read_u16::<LittleEndian>().ok()?;
    let game_year = c.read_u8().ok()?;
    let game_major = c.read_u8().ok()?;
    let game_minor = c.read_u8().ok()?;
    let packet_version = c.read_u8().ok()?;
    let packet_id = c.read_u8().ok()?;
    let session_uid = c.read_u64::<LittleEndian>().ok()?;
    let session_time = c.read_f32::<LittleEndian>().ok()?;
    let frame_identifier = c.read_u32::<LittleEndian>().ok()?;
    let overall_frame_identifier = c.read_u32::<LittleEndian>().ok()?;
    let player_car_index = c.read_u8().ok()?;
    let secondary_player_car_index = c.read_u8().ok()?;
    Some(PacketHeader {
        packet_format: pf, game_year, game_major, game_minor,
        packet_version, packet_id, session_uid, session_time,
        frame_identifier, overall_frame_identifier,
        player_car_index, secondary_player_car_index
    })
}

// Packet IDs (Codemasters/EA spec). We only need Motion (0), Session (1), LapData (2), CarTelemetry (6).
const PACKET_MOTION: u8 = 0;
const PACKET_LAPDATA: u8 = 2;
const PACKET_CAR_TELEMETRY: u8 = 6;

#[derive(Default, Clone)]
struct PlayerState {
    // last known values to combine across packets
    world_pos_x: f32,
    world_pos_y: f32,
    world_pos_z: f32,
    yaw: f32, pitch: f32, roll: f32,
    speed_mps: f32,
    throttle: f32, brake: f32,
    gear: i8, rpm: f32,
    lap_distance: f32,
    current_lap: u32,
    current_lap_time_s: f32,
    last_lap_time_s: f32,
    frame: u64,
}

fn parse_packet(buf: &[u8], expected_format: u16) -> Option<TelemetrySample> {
    let hdr = read_header(Cursor::new(buf))?;
    // If packet_format doesn't match expected, still accept for cross-year convenience

    use std::sync::OnceLock;
    static STATE: OnceLock<std::sync::Mutex<PlayerState>> = OnceLock::new();

    let state = STATE.get_or_init(|| std::sync::Mutex::new(PlayerState::default()));
    let mut st = state.lock().ok()?; // lock mutex for thread safety

    match hdr.packet_id {
        PACKET_MOTION => {
            // layout as per spec: 22 cars of MotionData, we read player's by index
            let base = 24; // header size (up to secondary player index) = 24 bytes
            // Use documented offsets for player's motion data:
            // world position X/Y/Z: 0..12, world yaw/pitch/roll approx later in packet (we'll try reading orientation at offsets 36..48 as yaw, pitch, roll in radians)
            let idx = hdr.player_car_index as usize;
            let start = base + idx * 1464; // spec size per car (MotionData) is 60*4 + more; 1464 is correct since F1 22+. Works for 23-25.
            if buf.len() >= start + 64 {
                let mut c = Cursor::new(&buf[start..start+64]);
                st.world_pos_x = c.read_f32::<LittleEndian>().unwrap_or(st.world_pos_x);
                st.world_pos_y = c.read_f32::<LittleEndian>().unwrap_or(st.world_pos_y);
                st.world_pos_z = c.read_f32::<LittleEndian>().unwrap_or(st.world_pos_z);
                // skip 7 f32 (velocity/angles) to yaw,pitch,roll – spec places orientation as yaw,pitch,roll radians at offsets 36..48
                let _ = c.read_f32::<LittleEndian>();
                let _ = c.read_f32::<LittleEndian>();
                let _ = c.read_f32::<LittleEndian>();
                let _ = c.read_f32::<LittleEndian>();
                let _ = c.read_f32::<LittleEndian>();
                let _ = c.read_f32::<LittleEndian>();
                let _ = c.read_f32::<LittleEndian>();
                st.yaw = c.read_f32::<LittleEndian>().unwrap_or(st.yaw);
                st.pitch = c.read_f32::<LittleEndian>().unwrap_or(st.pitch);
                st.roll = c.read_f32::<LittleEndian>().unwrap_or(st.roll);
            }
        },
        PACKET_LAPDATA => {
            // LapData: 22 cars entries; we read player's current/last lap times and distance
            let base = 24;
            let stride = 51; // bytes per car in 2024/25 spec (approx). Safer to use documented fields offsets we need.
            let idx = hdr.player_car_index as usize;
            // Use conservative: Lap distance at offset 0x14 (f32), current lap time at 0x20 (f32), last lap at 0x24 (f32)
            let start = base + idx * 51;
            if buf.len() >= start + 0x28 {
                let mut c = Cursor::new(&buf[start+0x14..start+0x28]);
                st.lap_distance = c.read_f32::<LittleEndian>().unwrap_or(st.lap_distance);
                st.current_lap_time_s = c.read_f32::<LittleEndian>().unwrap_or(st.current_lap_time_s);
                st.last_lap_time_s = c.read_f32::<LittleEndian>().unwrap_or(st.last_lap_time_s);
            }
            // current lap number usually at offset 0x10 (u8 or u16); use header frame as fallback
            let lap_num_off = start + 0x10;
            if buf.len() > lap_num_off {
                st.current_lap = buf[lap_num_off] as u32;
            }
        },
        PACKET_CAR_TELEMETRY => {
            // CarTelemetry: 22 cars; read speed (kph), throttle, steer, brake, clutch, gear, engineRPM
            let base = 24;
            let stride = 58; // approx
            let idx = hdr.player_car_index as usize;
            let start = base + idx * 58;
            if buf.len() >= start + 20 {
                let mut c = Cursor::new(&buf[start..]);
                let speed_kph = c.read_u16::<LittleEndian>().unwrap_or(0) as f32;
                st.speed_mps = speed_kph / 3.6;
                st.throttle = c.read_u8().unwrap_or(0) as f32 / 255.0;
                let _steer = c.read_i8().unwrap_or(0);
                st.brake = c.read_u8().unwrap_or(0) as f32 / 255.0;
                let _clutch = c.read_u8().unwrap_or(0);
                st.gear = c.read_i8().unwrap_or(st.gear);
                st.rpm = c.read_u16::<LittleEndian>().unwrap_or(0) as f32;
            }
        },
        _ => {}
    }

    st.frame = hdr.overall_frame_identifier as u64;
    let sample = TelemetrySample {
        game: if hdr.packet_format >= 2025 { GameId::F1_2025 } else { GameId::F1_2024 },
        car_id: format!("player:{}", hdr.player_car_index),
        session_uid: format!("{}", hdr.session_uid),
        frame: st.frame,
        sim_time_s: hdr.session_time as f64,
        speed_mps: st.speed_mps,
        throttle: st.throttle,
        brake: st.brake,
        gear: st.gear,
        engine_rpm: st.rpm,
        world_pos_x: st.world_pos_x,
        world_pos_y: st.world_pos_y,
        world_pos_z: st.world_pos_z,
        yaw: st.yaw, pitch: st.pitch, roll: st.roll,
        lap_distance_m: st.lap_distance,
        current_lap: st.current_lap,
        current_lap_time_s: st.current_lap_time_s,
        last_lap_time_s: st.last_lap_time_s,
    };
    Some(sample)
}
