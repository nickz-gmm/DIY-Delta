# Memory Management and Vector Allocation Optimizations

## Overview
This PR refactors the Delta telemetry analysis application to optimize memory usage and vector allocations, significantly improving performance when processing large collections of telemetry data.

## Key Optimizations Made

### 1. Eliminated Excessive Lap Object Cloning
**Problem**: The commands.rs file was cloning entire Lap objects (which contain Vec<TelemetryPoint> with potentially thousands of points) unnecessarily.

**Solution**: Modified functions to work with references instead:
- `analyze_laps`: Changed from `Vec<Lap>` to `Vec<&Lap>` (eliminates copying entire lap data)
- `build_track_map`: Pass `&Lap` instead of cloning 
- `export_file`: Use `Vec<&Lap>` instead of `Vec<Lap>`

**Impact**: Reduces memory usage by orders of magnitude when processing multiple laps, as each Lap can contain thousands of TelemetryPoint objects.

### 2. Added Vector Capacity Pre-allocation
**Problem**: Multiple functions were using `Vec::new()` for vectors with predictable sizes, causing multiple reallocations as they grow.

**Solutions**:
- `overlay_speed_vs_distance`: Pre-allocate with `((max_len / step) + 1)` capacity
- `lap_summary`: Pre-allocate sector_times_ms with `laps.len() * 3` capacity  
- `rolling_delta_vs_reference`: Pre-allocate with calculated row count
- `build_track_map`: Pre-allocate polyline with `lap.points.len()` and corners with `peaks.len()`
- `per_corner_metrics`: Pre-allocate with `peaks.len()` capacity
- `auto_sectors`: Pre-allocate idx vector with `curv.len()` capacity

**Impact**: Eliminates multiple memory reallocations and copying during vector growth.

### 3. Reduced String Cloning in Export Functions
**Problem**: CSV and MoTeC export functions were cloning the same metadata strings (game, car, track) for every telemetry point within a lap.

**Solution**: Cache string references outside the inner loop and reuse them:
```rust
// Before: l.meta.game.clone() for every point
// After: Cache once per lap
let game = &l.meta.game;
let car = &l.meta.car; 
let track = &l.meta.track;
```

**Impact**: Reduces string allocations from O(points) to O(laps) complexity.

### 4. Optimized Loop Structures
**Problem**: Some functions used inefficient iterator chains that created intermediate collections.

**Solution**: 
- `build_track_map`: Changed from `.iter().map().collect()` to explicit loop with pre-allocated Vec
- `auto_sectors`: Explicit loop instead of `.iter().enumerate().map().collect()`

## Performance Benefits

1. **Memory Usage**: Dramatically reduced memory consumption by eliminating unnecessary Lap clones
2. **Allocations**: Fewer memory allocations due to proper capacity hints
3. **CPU**: Reduced time spent on memory copying and reallocation
4. **Scalability**: Performance improvement scales with data size (more laps = bigger benefit)

## Testing
- Added comprehensive unit tests to verify optimized functions work correctly
- All existing functionality preserved
- Build system updated with proper .gitignore to exclude build artifacts

## Files Modified
- `apps/desktop/src-tauri/src/commands.rs`: Eliminated Lap cloning
- `crates/analysis/src/lib.rs`: Added capacity hints and optimized algorithms
- `crates/io/src/lib.rs`: Reduced string cloning in export functions
- `crates/analysis/Cargo.toml`: Added uuid dependency for tests
- `.gitignore`: Added to exclude build artifacts

## Backward Compatibility
All changes are internal optimizations that maintain the same public API. No breaking changes to function signatures or behavior.