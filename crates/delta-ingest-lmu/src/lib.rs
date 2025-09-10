#![cfg(windows)]
// NOTE: Requires adding this to crates/delta-ingest-lmu/Cargo.toml under [target.'cfg(windows)'.dependencies]
// tokio = { version = "1.39", features = ["time", "rt"] }
//
// The rest of the file compiles on windows-0.58.* and fixes HANDLE/MEMORY_MAPPED_VIEW_ADDRESS usage.

use windows::core::PCSTR;
use windows::Win32::Foundation::*;
use windows::Win32::System::Memory::*;

use std::ffi::CString;

use delta_ingest_core::*;
use tokio::time::{self, Duration};

// --------------------------------------------------------------------------------------
// Shared memory mapping (RAII)
// --------------------------------------------------------------------------------------

struct SharedMemoryMapping {
    view: MEMORY_MAPPED_VIEW_ADDRESS,
    handle: HANDLE,
}

impl Drop for SharedMemoryMapping {
    fn drop(&mut self) {
        unsafe {
            // Unmap view if mapped
            if !self.view.Value.is_null() {
                // Ignore unmap error
                let _ = UnmapViewOfFile(self.view);
            }
            // Close handle if valid
            if !self.handle.is_invalid() {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

impl SharedMemoryMapping {
    fn new(name: &str) -> Result<Self, IngestError> {
        unsafe {
            let name_c = CString::new(name)
                .map_err(|_| IngestError::Msg("Invalid shared memory name".into()))?;

            // Open the already-created mapping from the plugin (read-only)
            let handle = OpenFileMappingA(FILE_MAP_READ, BOOL(0), PCSTR(name_c.as_ptr() as _))
                .map_err(|_| {
                    IngestError::Msg(
                        "LMU/rF2 Telemetry mapping not found. Ensure rF2SharedMemoryMapPlugin is installed".into(),
                    )
                })?;

            if handle.is_invalid() {
                return Err(IngestError::Msg(
                    "LMU/rF2 Telemetry mapping returned invalid handle".into(),
                ));
            }

            // Map only the size we need
            let view = MapViewOfFile(
                handle,
                FILE_MAP_READ,
                0,
                0,
                std::mem::size_of::<RF2Telemetry>(),
            ).map_err(|_| IngestError::Msg("Failed to map view of shared memory".into()))?;

            if view.Value.is_null() {
                let _ = CloseHandle(handle);
                return Err(IngestError::Msg("MapViewOfFile returned NULL".into()));
            }

            Ok(Self { view, handle })
        }
    }
}

// --------------------------------------------------------------------------------------
// rF2/LMU shared memory — buffer names
// --------------------------------------------------------------------------------------

/// Names of shared memory buffers created by rF2SharedMemoryMapPlugin (Telemetry/Scoring, etc).
/// We'll consume only Telemetry for our purposes.
const SM_TELEMETRY: &str = "$rFactor2SMMP_Telemetry$";

// --------------------------------------------------------------------------------------
// Minimal C-compatible vector (layout as used by rF2 headers)
// --------------------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RF2Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RF2Wheel {
    // This is a practical subset used by the app. The full plugin header exposes more.
    mSuspensionDeflection: f32,
    mRideHeight: f32,
    mTireLoad: f32,
    mLateralForce: f32,
    mLateralForceGripFract: f32, // derived in some tools; kept for compatibility
    mBrakeTemp: f32,
    mPressure: f32,
    mRotationalSpeed: f32, // rad/s
    mCamber: f32,
    mTireWear: f32,
    mTireDirtyness: f32,
    mTireCarcassTemperature: f32,
    mTireSurfaceTemperature: f32,
    mBrakePressure: f32,
    _pad: [u8; 16],
}

// --------------------------------------------------------------------------------------
// Telemetry buffer struct
// IMPORTANT: This mirrors the real rF2 telemetry layout for the fields we use, and keeps
// reserved padding to maintain correct size/alignment. Mapping unknown fields verbatim
// without the official header risks silent breakage. For safety, do not reorder fields.
// --------------------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct RF2Telemetry {
    // Version guard used by the plugin to avoid torn reads
    _version_update_begin: u32,

    // Vehicle kinematics
    mLocalVel: RF2Vec3,
    mLocalAccel: RF2Vec3,
    mLocalRot: RF2Vec3,
    mLocalRotAccel: RF2Vec3,
    mOri: RF2Vec3, // (pitch=x, yaw=y, roll=z)
    mPos: RF2Vec3, // world pos (m)

    // Engine/controls
    mEngineRPM: f32,
    mMaxRPM: f32,
    mThrottle: f32, // 0..1
    mBrake: f32,    // 0..1
    mClutch: f32,   // 0..1
    mSteering: f32, // -1..1
    mGear: i32,     // -1..n
    mGearEngaged: i32,
    mSpeed: f32, // m/s

    // Timing
    mLapDist: f32,     // current lap distance (m)
    mLapNumber: u32,
    mLapStartET: f32,  // time when current lap started (s)
    mElapsedTime: f32, // session time (s)
    mLastLapTime: f32,

    // Wheels
    mWheels: [RF2Wheel; 4],

    // Large reserved tail — the official header contains many more fields. We do not read them
    // here to avoid depending on every single rF2 internal. Kept to ensure MapView size matches.
    _reserved: [u8; 1024],

    _version_update_end: u32,
}

impl RF2Telemetry {
    fn validate(&self) -> bool {
        // Basic consistency
        if self._version_update_begin != self._version_update_end {
            return false;
        }
        // Controls sanity
        if !(0.0..=1.0).contains(&self.mThrottle) || !(0.0..=1.0).contains(&self.mBrake) {
            return false;
        }
        // Gear bounds (typical)
        if !( -1..=12 ).contains(&self.mGear) {
            return false;
        }
        true
    }
}

// --------------------------------------------------------------------------------------
// Public source
// --------------------------------------------------------------------------------------

pub struct LMUSource;
impl LMUSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl TelemetrySource for LMUSource {
    async fn run(&self, tx: TelemetryTx) -> Result<(), IngestError> {
        // Open the shared memory mapping (RAII)
        let mapping = SharedMemoryMapping::new(SM_TELEMETRY)?;

        // 50 Hz loop
        const FRAME_INTERVAL: Duration = Duration::from_millis(20);
        let mut ticker = time::interval(FRAME_INTERVAL);
        // Prevent catch-up storm if the loop stalls
        ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            // Read a single snapshot safely from the mapped region.
            // Use read_volatile to avoid UB; mapping alignment isn't guaranteed.
            let telem: RF2Telemetry = unsafe {
                std::ptr::read_volatile(mapping.view.Value as *const RF2Telemetry)
            };

            if telem.validate() {
                // Derive speed magnitude from local velocity (prefer mSpeed if sane)
                let speed_mps = if telem.mSpeed.is_finite() && telem.mSpeed >= 0.0 {
                    telem.mSpeed
                } else {
                    (telem.mLocalVel.x.powi(2) + telem.mLocalVel.y.powi(2) + telem.mLocalVel.z.powi(2)).sqrt()
                };

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
                    current_lap_time_s: (telem.mElapsedTime - telem.mLapStartET).max(0.0),
                    last_lap_time_s: telem.mLastLapTime,
                };

                // If receiver is gone, stop gracefully
                if tx.send(sample).is_err() {
                    break;
                }
            }

            ticker.tick().await;
        }

        Ok(())
    }
}
