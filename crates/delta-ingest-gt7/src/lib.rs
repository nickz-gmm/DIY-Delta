\
use anyhow::Context;
use tokio::{net::UdpSocket, time::{self, Duration, Instant}};
use delta_ingest_core::{*, Game as GameId};
use salsa20::cipher::{KeyIvInit, StreamCipher};
use salsa20::Salsa20;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct GT7Config {
    /// Local bind address for receiving packets from the PS5 (default port 33740)
    pub bind_addr: String,
    /// PS5 console IP address to send heartbeat packets to
    pub console_ip: String,
    /// Packet variant to request via heartbeat: 'A', 'B', or '~'
    pub packet_variant: char,
}

impl Default for GT7Config {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:33740".into(),
            console_ip: "192.168.1.100".into(),
            packet_variant: 'A',
        }
    }
}

pub struct GT7Source { cfg: GT7Config }
impl GT7Source { pub fn new(cfg: GT7Config) -> Self { Self { cfg } } }

#[async_trait::async_trait]
impl TelemetrySource for GT7Source {
    async fn run(&self, tx: TelemetryTx) -> Result<(), IngestError> {
        let socket = UdpSocket::bind(&self.cfg.bind_addr).await
            .with_context(|| format!("bind {}", self.cfg.bind_addr))?;
       socket.connect((self.cfg.console_ip.as_str(), 33740))
    .await
    .with_context(|| format!("connect {}", self.cfg.console_ip))?;
        
        // heartbeat: a single ASCII byte indicating variant, repeated ~1s
        let variant = self.cfg.packet_variant as u8;
        let hb = vec![variant];
        let mut hb_interval = time::interval(Duration::from_millis(800));
        hb_interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

        let mut buf = vec![0u8; 1024];
        loop {
            tokio::select! {
                _ = hb_interval.tick() => {
                    let _ = socket.send(&hb).await;
                }
                Ok((len, _)) = socket.recv_from(&mut buf) => {
                    if let Some(sample) = decrypt_and_parse(&buf[..len], self.cfg.packet_variant) {
                        let _ = tx.send(sample);
                    }
                }
            }
        }
    }
}

// Encryption per community docs: Salsa20 with fixed key string and per-packet nonce bytes (0x40..0x47) derived by XORing a 32-bit constant depending on variant.
fn decrypt_and_parse(pkt: &[u8], variant: char) -> Option<TelemetrySample> {
    if pkt.len() < 64 { return None; }

    // Key (32 bytes) — from GT7 "Simulator Interface Packet GT7 ver 0.0", truncated/padded.
    let mut key = [0u8; 32];
    let key_str = b"Simulator Interface Packet GT7 ver 0.0";
    let copy_len = key_str.len().min(32);
    key[..copy_len].copy_from_slice(&key_str[..copy_len]);

    // Nonce (8 bytes) at 0x40..0x47, but first 4 bytes are XOR'ed with variant-specific constant
    let mut nonce = [0u8; 8];
    nonce.copy_from_slice(&pkt[0x40..0x48]);
    let xconst: u32 = match variant {
        'A' => 0xDEADBEAF, // variant A (as per community docs)
        'B' => 0xDEADBEEF, // variant B
        _ => 0x54_5F_4C_7E, // fallback for "~" variant (placeholder constant)
    };
    let mut first4 = u32::from_le_bytes(nonce[0..4].try_into().unwrap());
    first4 ^= xconst;
    nonce[0..4].copy_from_slice(&first4.to_le_bytes());

    // Salsa20 uses 8-byte nonce; construct cipher and decrypt in-place after 0x48
    let iv = &nonce;
    let mut cipher = Salsa20::new((&key).into(), iv.into());
    let mut payload = pkt[0x48..].to_vec();
    cipher.apply_keystream(&mut payload);

    // Packet A structure (296 bytes). We'll parse a few key fields by offsets known from documentation.
    // Offsets (little endian):
    // 0x00: sequence (u32)
    // 0x04: magic/game id (u32)
    // 0x08: time_ms (u32)
    // 0x10: pos_x (f32), 0x14: pos_y (f32), 0x18: pos_z (f32)
    // 0x1C: yaw (f32), 0x20: pitch (f32), 0x24: roll (f32)
    // 0x40: speed_kmh (f32)
    // 0x44: engine_rpm (f32)
    // 0x48: throttle (f32 0..1)
    // 0x4C: brake (f32 0..1)
    // 0x50: gear (i32) -1..n
    if payload.len() < 0x60 { return None; }
    let mut c = Cursor::new(&payload);

    let _seq = c.read_u32::<LittleEndian>().ok()?;
    let _magic = c.read_u32::<LittleEndian>().ok()?;
    let time_ms = c.read_u32::<LittleEndian>().ok()?;
    // skip 4 bytes (unknown)
    let _ = c.read_u32::<LittleEndian>().ok()?;

    // Positions and orientation
    let pos_x = c.read_f32::<LittleEndian>().ok()?;
    let pos_y = c.read_f32::<LittleEndian>().ok()?;
    let pos_z = c.read_f32::<LittleEndian>().ok()?;
    let yaw = c.read_f32::<LittleEndian>().ok()?;
    let pitch = c.read_f32::<LittleEndian>().ok()?;
    let roll = c.read_f32::<LittleEndian>().ok()?;

    // Skip to dynamics (0x40)
    let dyn_off = 0x40usize;
    let mut d = Cursor::new(&payload[dyn_off..]);
    let speed_kmh = d.read_f32::<LittleEndian>().ok()?;
    let engine_rpm = d.read_f32::<LittleEndian>().ok()?;
    let throttle = d.read_f32::<LittleEndian>().ok()?;
    let brake = d.read_f32::<LittleEndian>().ok()?;
    let gear_i32 = d.read_i32::<LittleEndian>().ok()?;

    Some(TelemetrySample {
        game: GameId::GT7,
        car_id: "player:0".into(),
        session_uid: "gt7".into(),
        frame: time_ms as u64,
        sim_time_s: (time_ms as f64) / 1000.0,
        speed_mps: speed_kmh / 3.6,
        throttle,
        brake,
        gear: gear_i32 as i8,
        engine_rpm,
        world_pos_x: pos_x, world_pos_y: pos_y, world_pos_z: pos_z,
        yaw, pitch, roll,
        lap_distance_m: 0.0,
        current_lap: 0,
        current_lap_time_s: 0.0,
        last_lap_time_s: 0.0,
    })
}
