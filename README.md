# Delta — Full UI + Native Game Connectors

This repo integrates the **Delta** desktop UI (Tauri + React) with **live, non‑stubbed connectors** for:
- **F1 24/25 (UDP)** — PC/console with UDP enabled
- **Gran Turismo 7 (PS5)** — UDP with Salsa20 decryption + heartbeat
- **Le Mans Ultimate (Windows)** — via rFactor2 Shared Memory

It also wires **MoTeC‑CSV/CSV/NDJSON import/export**, **multi‑lap overlays**, **time‑delta ribbon**, **track map with auto corners/sectors**, **per‑corner metrics**, **consistency**, and **workspaces/notes**.

## Build
- Prereqs: Node 18+, Rust stable, Tauri deps (Xcode CLT on macOS, MSVC on Windows)
```bash
cd apps/desktop
pnpm install
pnpm tauri dev   # dev
pnpm tauri build # release
```

## Live sources
- F1 24/25: enable UDP in-game, set Format 2024/2025, target IP and port 20777 (default). Start from **Dashboard → Start F1**.
- GT7: enable UDP “Data Out” in GT7, enter your PS5 IP, choose variant (A/B/~). Start from **Dashboard → Start GT7**.
- LMU (Windows): install and enable the rF2 Shared Memory Map plugin; start from **Dashboard → Start LMU**.

> Lap building: when games don’t provide lap distance/number (e.g., GT7), Delta estimates lap distance from XY path and auto-detects laps by re‑crossing the start area after a minimum elapsed time.

## Storage & I/O
- Import CSV or NDJSON via command.
- Export CSV, NDJSON, and MoTeC‑compatible CSV for MoTeC i2.
- Workspaces saved under OS data dir (`Delta/workspaces`).
