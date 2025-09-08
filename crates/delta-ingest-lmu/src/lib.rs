
#![cfg(windows)]
use anyhow::Context;
use windows::Win32::System::Memory::*;
use windows::Win32::Foundation::*;
use std::ffi::CString;
use std::ptr::null_mut;
use std::time::Duration;
use std::thread;
use delta_ingest_core::*;

/// Names of shared memory buffers created by rF2SharedMemoryMapPlugin (Telemetry/Scoring, etc).
/// We'll consume only Telemetry for our purposes.
const SM_TELEMETRY: &str = "$rFactor2SMMP_Telemetry$";

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RF2Vec3 { x: f32, y: f32, z: f32 }

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RF2Telemetry {
    // This struct is a reduced view of the plugin's rF2Telemetry buffer layout.
    // Only a subset of fields needed by Delta are mapped here, with conservative ordering.
    _version_update_begin: u32, // version check (begin)
    // ...
    // Vehicle kinematics
    mLocalVel: RF2Vec3,       // local velocity (m/s)
    mLocalAccel: RF2Vec3,     // local acceleration
    mOri: RF2Vec3,            // orientation (rad): pitch (x), yaw (y), roll (z)  [order differs in plugin, we remap when publishing]
    mPos: RF2Vec3,            // world pos (m)
    // Engine/controls
    mEngineRPM: f32,
    mMaxRPM: f32,
    mThrottle: f32, // 0..1
    mBrake: f32,    // 0..1
    mClutch: f32,   // 0..1
    mSteering: f32, // -1..1
    mGear: i32,     // -1..n
    // Timing
    mLapDist: f32,      // current lap distance (m)
    mLapNumber: u32,
    mLapStartET: f32,   // time when current lap started
    mElapsedTime: f32,  // session time
    mLastLapTime: f32,
    _reserved: [u8; 512],
    _version_update_end: u32, // version check (end)
}

pub struct LMUSource;
impl LMUSource { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl TelemetrySource for LMUSource {
    async fn run(&self, tx: TelemetryTx) -> Result<(), IngestError> {
        unsafe {
            // Open an existing named mapping
            let name = CString::new(SM_TELEMETRY).unwrap();
            let hmap = OpenFileMappingA(FILE_MAP_READ, BOOL(0), PCSTR(name.as_ptr() as _))
    .map_err(|_| IngestError::Msg("LMU/rF2 Telemetry mapping not found. Ensure the rF2SharedMemoryMapPlugin is installed and enabled.".into()))?;

if hmap.is_invalid() {
    return Err(IngestError::Msg("LMU/rF2 Telemetry mapping not found. Ensure the rF2SharedMemoryMapPlugin is installed and enabled.".into()));
}
let view = MapViewOfFile(hmap, FILE_MAP_READ, 0, 0, std::mem::size_of::<RF2Telemetry>());
if view.is_null() {
    CloseHandle(hmap);
    return Err(IngestError::Msg("Failed to map LMU Telemetry view".into()));
}
            let mut last_frame: u64 = 0;
            loop {
                let telem: RF2Telemetry = std::ptr::read(view as *const RF2Telemetry);
                // Basic sanity: version markers equal
                if telem._version_update_begin != telem._version_update_end {
                    // Torn frame; skip and yield
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
                // Derive speed magnitude from local velocity
                let speed_mps = (telem.mLocalVel.x.powi(2) + telem.mLocalVel.y.powi(2) + telem.mLocalVel.z.powi(2)).sqrt();

                let sample = TelemetrySample {
                    game: Game::LMU,
                    car_id: "player:0".to_string(),
                    session_uid: "lmu".to_string(),
                    frame: (telem.mElapsedTime * 1000.0) as u64,
                    sim_time_s: telem.mElapsedTime as f64,
                    speed_mps,
                    throttle: telem.mThrottle,
                    brake: telem.mBrake,
                    gear: telem.mGear as i8,
                    engine_rpm: telem.mEngineRPM,
                    world_pos_x: telem.mPos.x,
                    world_pos_y: telem.mPos.y,
                    world_pos_z: telem.mPos.z,
                    // plugin stores orientation as (pitch, yaw, roll). Publish yaw,pitch,roll.
                    yaw: telem.mOri.y,
                    pitch: telem.mOri.x,
                    roll: telem.mOri.z,
                    lap_distance_m: telem.mLapDist,
                    current_lap: telem.mLapNumber,
                    current_lap_time_s: telem.mElapsedTime - telem.mLapStartET,
                    last_lap_time_s: telem.mLastLapTime,
                };
                let _ = tx.send(sample);
                thread::sleep(Duration::from_millis(20)); // ~50 Hz
            }
        }
    }
}
