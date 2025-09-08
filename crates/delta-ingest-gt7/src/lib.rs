use anyhow::Context;
use tokio::{net::UdpSocket, time};
use std::time::Duration;

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
        let socket = UdpSocket::bind(&self.cfg.bind_addr)
            .await
            .with_context(|| format!("bind {}", self.cfg.bind_addr))?;

        // We "connect" the UDP socket so send()/recv() go to/from this peer by default.
        socket.connect((&*self.cfg.console_ip, 33740))
            .await
            .with_context(|| format!("connect {}", self.cfg.console_ip))?;

        // Heartbeat: single ASCII byte indicating variant, ~every 0.8s
        let variant = normalise_variant(self.cfg.packet_variant);
        let hb = [variant as u8];

        let mut hb_interval = time::interval(Duration::from_millis(800));
        // If we miss ticks (app is busy), don't try to "catch up"
        hb_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        let mut buf = vec![0u8; 2048];

        loop {
            tokio::select! {
                _ = hb_interval.tick() => {
                    let _ = socket.send(&hb).await; // best-effort
                }
                recv = socket.recv(&mut buf) => {
                    match recv {
                        Ok(len) => {
                            if let Some(sample) = decrypt_and_parse(&buf[..len], variant) {
                                if tx.send(sample).is_err() {
                                    // receiver dropped; time to stop
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            // Surface the error – breaks the run loop and returns an error
                            return Err(IngestError::Other(e.into()));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[inline]
fn normalise_variant(v: char) -> char {
    match v {
        'A' | 'B' | '~' => v,
        _ => 'A',
    }
}

// Encryption per community docs: Salsa20 with fixed key string and per-packet nonce
// bytes (0x40..0x47) whose first 4 bytes are XOR'd with a variant-specific constant.
fn decrypt_and_parse(pkt: &[u8], variant: char) -> Option<TelemetrySample> {
    // Header needs at least up to nonce at 0x40..0x47 and some payload.
    if pkt.len() < 0x48 { return None; }

    // Key (32 bytes) — "Simulator Interface Packet GT7 ver 0.0" (padded/truncated)
    let mut key = [0u8; 32];
    let key_str = b"Simulator Interface Packet GT7 ver 0.0";
    let copy_len = key_str.len().min(key.len());
    key[..copy_len].copy_from_slice(&key_str[..copy_len]);

    // Nonce (8 bytes) at 0x40..0x47; first 4 bytes XORed with variant constant
    let mut nonce = [0u8; 8];
    nonce.copy_from_slice(&pkt[0x40..0x48]);
    let xconst: u32 = match variant {
        'A' => 0xDEAD_BEAF, // community value for A (placeholder if you have the exact one)
        'B' => 0xDEAD_BEEF, // community value for B
        _   => 0x545F_4C7E, // "~" fallback placeholder
    };
    let mut first4 = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
    first4 ^= xconst;
    nonce[0..4].copy_from_slice(&first4.to_le_bytes());

    // Decrypt payload after 0x48
    if pkt.len() <= 0x48 { return None; }
    let mut payload = pkt[0x48..].to_vec();

    // Salsa20 uses 32-byte key + 8-byte nonce
    let mut cipher = Salsa20::new((&key).into(), (&nonce).into());
    cipher.apply_keystream(&mut payload);

    // Packet A structure (approx 296 bytes). Read a minimal set with bounds checks.
    if payload.len() < 0x60 { return None; }
    let mut c = Cursor::new(&payload);

    let _seq   = c.read_u32::<LittleEndian>().ok()?;
    let _magic = c.read_u32::<LittleEndian>().ok()?;
    let time_ms = c.read_u32::<LittleEndian>().ok()?;
    let _unknown = c.read_u32::<LittleEndian>().ok()?; // skip

    // Positions and orientation (x,y,z,yaw,pitch,roll)
    let pos_x = c.read_f32::<LittleEndian>().ok()?;
    let pos_y = c.read_f32::<LittleEndian>().ok()?;
    let pos_z = c.read_f32::<LittleEndian>().ok()?;
    let yaw   = c.read_f32::<LittleEndian>().ok()?;
    let pitch = c.read_f32::<LittleEndian>().ok()?;
    let roll  = c.read_f32::<LittleEndian>().ok()?;

    // Dynamics block starting at 0x40
    const DYN_OFF: usize = 0x40;
    if payload.len() < DYN_OFF + 0x14 { return None; }
    let mut d = Cursor::new(&payload[DYN_OFF..]);
    let speed_kmh = d.read_f32::<LittleEndian>().ok()?;
    let engine_rpm = d.read_f32::<LittleEndian>().ok()?;
    let throttle = d.read_f32::<LittleEndian>().ok()?;
    let brake    = d.read_f32::<LittleEndian>().ok()?;
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

        world_pos_x: pos_x,
        world_pos_y: pos_y,
        world_pos_z: pos_z,
        yaw, pitch, roll,

        // Not present in this packet; can be derived in a higher layer if needed.
        lap_distance_m: 0.0,
        current_lap: 0,
        current_lap_time_s: 0.0,
        last_lap_time_s: 0.0,
    })
}
