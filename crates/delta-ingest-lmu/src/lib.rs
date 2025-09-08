#![cfg(windows)]
use windows::Win32::Foundation::*;
use windows::Win32::System::Memory::*;

use std::ffi::CString;
use std::ptr::null_mut;

use delta_ingest_core::*;
use tokio::time::{self, Duration, Instant};

struct SharedMemoryMapping {
    view: *mut std::ffi::c_void,
    handle: HANDLE,
}

impl Drop for SharedMemoryMapping {
    fn drop(&mut self) {
        unsafe {
            if !self.view.is_null() {
                UnmapViewOfFile(self.view);
            }
            if !self.handle.is_invalid() {
                // Ignore failure on close
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

impl SharedMemoryMapping {
    fn new(name: &str) -> Result<Self, IngestError> {
        unsafe {
            let name_c =
                CString::new(name).map_err(|_| IngestError::Msg("Invalid shared memory name".into()))?;

            // Open the already-created mapping from the plugin (read-only)
            let handle = OpenFileMappingA(FILE_MAP_READ.0, BOOL(0), PCSTR(name_c.as_ptr() as _));
            if handle.is_invalid() {
                return Err(IngestError::Msg(
                    "LMU/rF2 Telemetry mapping not found. Ensure rF2SharedMemoryMapPlugin is installed".into(),
                ));
            }

            // Map only the size we need
            let view = MapViewOfFile(
                handle,
                FILE_MAP_READ.0,
                0,
                0,
                std::mem::size_of::<RF2Telemetry>(),
            );
            if view.is_null() {
                let _ = CloseHandle(handle);
                return Err(IngestError::Msg("Failed to map view of shared memory".into()));
            }

            Ok(Self { view, handle })
        }
    }
}

/// Names of shared memory buffers created by rF2SharedMemoryMapPlugin (Telemetry/Scoring, etc).
/// We'll consume only Telemetry for our purposes.
const SM_TELEMETRY: &str = "$rFactor2SMMP_Telemetry$";

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RF2Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RF2Telemetry {
    // Reduced view of the plugin's rF2Telemetry buffer layout.
    _version_update_begin: u32, // version check (begin)
    // ...
    // Vehicle kinematics
    mLocalVel: RF2Vec3, // local velocity (m/s)
    mLocalAccel: RF2Vec3,
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
    // Timing
    mLapDist: f32,     // current lap distance (m)
    mLapNumber: u32,
    mLapStartET: f32,  // time when current lap started
    mElapsedTime: f32, // session time
    mLastLapTime: f32,
    _reserved: [u8; 512],
    _version_update_end: u32, // version check (end)
}

impl RF2Telemetry {
    fn validate(&self) -> bool {
        // Basic consistency
        if self._version_update_begin != self._version_update_end {
            return false;
        }
        // Physics sanity
        let ok_vel = |v: f32| v.is_finite() && (-1000.0..=1000.0).contains(&v);
        if !ok_vel(self.mLocalVel.x) || !ok_vel(self.mLocalVel.y) || !ok_vel(self.mLocalVel.z) {
            return false;
        }
        // Controls sanity
        if !(0.0..=1.0).contains(&self.mThrottle)
            || !(0.0..=1.0).contains(&self.mBrake)
            || !(-1..=8).contains(&self.mGear)
        {
            return false;
        }
        true
    }
}

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
            // Use read_unaligned or read_volatile to avoid UB; mapping alignment isn't guaranteed.
            let telem: RF2Telemetry = unsafe {
                // std::ptr::read_unaligned(mapping.view as *const RF2Telemetry)
                std::ptr::read_volatile(mapping.view as *const RF2Telemetry)
            };

            if telem.validate() {
                // Derive speed magnitude from local velocity
                let speed_mps = (telem.mLocalVel.x.powi(2)
                    + telem.mLocalVel.y.powi(2)
                    + telem.mLocalVel.z.powi(2))
                .sqrt();

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

            // Wait for next tick (non-blocking)
            ticker.tick().await;
        }

        Ok(())
    }
}
