import { useCallback, useEffect, useMemo, useState } from 'react'
import { startF1, startGT7, startLMU, stopAll, listLaps } from '../lib/api'

type F1Format = 2024 | 2025
type GT7Variant = 'A' | 'B' | '~'

type Lap = {
  id: string
  game: string
  track: string
  car: string
  time_ms: number
  lap_number: number
}

export default function Dashboard() {
  const [port, setPort] = useState<number>(20777)
  const [format, setFormat] = useState<F1Format>(2025)
  const [consoleIp, setConsoleIp] = useState<string>('192.168.1.100')
  const [variant, setVariant] = useState<GT7Variant>('A')
  const [laps, setLaps] = useState<Lap[]>([])
  const [busy, setBusy] = useState<'f1' | 'gt7' | 'lmu' | 'stop' | null>(null)
  const [error, setError] = useState<string | null>(null)

  const fmtLap = useCallback((ms: number) => {
    if (!Number.isFinite(ms)) return '-'
    const s = ms / 1000
    return `${s.toFixed(3)}s`
  }, [])

  const refresh = useCallback(async () => {
    try {
      setError(null)
      const items = await listLaps()
      // Basic runtime type narrowing
      const safe = Array.isArray(items) ? (items as Lap[]) : []
      setLaps(safe)
    } catch (e: any) {
      setError(e?.message ?? 'Failed to load laps')
      setLaps([])
    }
  }, [])

  useEffect(() => {
    void refresh()
  }, [refresh])

  const handleStartF1 = useCallback(async () => {
    try {
      setBusy('f1'); setError(null)
      await startF1(port, format)
    } catch (e: any) {
      setError(e?.message ?? 'Failed to start F1')
    } finally {
      setBusy(null)
    }
  }, [port, format])

  const handleStartGT7 = useCallback(async () => {
    try {
      setBusy('gt7'); setError(null)
      await startGT7(consoleIp.trim(), variant)
    } catch (e: any) {
      setError(e?.message ?? 'Failed to start GT7')
    } finally {
      setBusy(null)
    }
  }, [consoleIp, variant])

  const handleStartLMU = useCallback(async () => {
    try {
      setBusy('lmu'); setError(null)
      await startLMU()
    } catch (e: any) {
      setError(e?.message ?? 'Failed to start LMU')
    } finally {
      setBusy(null)
    }
  }, [])

  const handleStopAll = useCallback(async () => {
    try {
      setBusy('stop'); setError(null)
      await stopAll()
    } catch (e: any) {
      setError(e?.message ?? 'Failed to stop sources')
    } finally {
      setBusy(null)
    }
  }, [])

  const formatOptions = useMemo<F1Format[]>(() => [2024, 2025], [])

  // Input guards
  const onPortChange = (v: string) => {
    const n = Number.parseInt(v, 10)
    setPort(Number.isFinite(n) && n > 0 && n <= 65535 ? n : 20777)
  }

  const onFormatChange = (v: string) => {
    const n = Number.parseInt(v, 10) as F1Format
    setFormat(n === 2024 || n === 2025 ? n : 2025)
  }

  const onVariantChange = (v: string) => {
    const vv = (v as GT7Variant)
    setVariant(vv === 'A' || vv === 'B' || vv === '~' ? vv : 'A')
  }

  return (
    <div className="space-y-6">
      <section className="bg-panel/60 rounded-2xl p-5 shadow-soft border border-white/5">
        <h2 className="text-xl font-semibold mb-2">Live Sources</h2>

        {error && (
          <div className="mb-3 text-sm text-red-400">
            {error}
          </div>
        )}

        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          {/* F1 */}
          <div className="p-4 rounded-2xl bg-white/5 border border-white/5 space-y-2">
            <div className="font-semibold">F1 24/25</div>
            <div className="text-sm text-white/60">UDP</div>
            <div className="flex gap-2">
              <input
                className="bg-white/5 rounded-xl px-3 py-2 w-28"
                type="number"
                inputMode="numeric"
                value={port}
                onChange={(e) => onPortChange(e.target.value)}
              />
              <select
                className="bg-white/5 rounded-xl px-3 py-2"
                value={format}
                onChange={(e) => onFormatChange(e.target.value)}
              >
                {formatOptions.map((yr) => (
                  <option key={yr} value={yr}>{yr}</option>
                ))}
              </select>
            </div>
            <button
              type="button"
              className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30 disabled:opacity-50"
              onClick={handleStartF1}
              disabled={busy !== null}
            >
              {busy === 'f1' ? 'Starting…' : 'Start F1'}
            </button>
          </div>

          {/* GT7 */}
          <div className="p-4 rounded-2xl bg-white/5 border border-white/5 space-y-2">
            <div className="font-semibold">Gran Turismo 7</div>
            <div className="text-sm text-white/60">PS5 UDP (Salsa20)</div>
            <div className="flex gap-2">
              <input
                className="bg-white/5 rounded-xl px-3 py-2"
                value={consoleIp}
                onChange={(e) => setConsoleIp(e.target.value)}
                placeholder="PS5 IP"
                inputMode="numeric"
              />
              <select
                className="bg-white/5 rounded-xl px-3 py-2"
                value={variant}
                onChange={(e) => onVariantChange(e.target.value)}
              >
                <option value="A">A</option>
                <option value="B">B</option>
                <option value="~">~</option>
              </select>
            </div>
            <button
              type="button"
              className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30 disabled:opacity-50"
              onClick={handleStartGT7}
              disabled={busy !== null}
            >
              {busy === 'gt7' ? 'Starting…' : 'Start GT7'}
            </button>
          </div>

          {/* LMU */}
          <div className="p-4 rounded-2xl bg-white/5 border border-white/5 space-y-2">
            <div className="font-semibold">Le Mans Ultimate</div>
            <div className="text-sm text-white/60">rF2 Shared Memory (Windows)</div>
            <button
              type="button"
              className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30 disabled:opacity-50"
              onClick={handleStartLMU}
              disabled={busy !== null}
            >
              {busy === 'lmu' ? 'Starting…' : 'Start LMU'}
            </button>
          </div>
        </div>

        <div className="pt-4 flex items-center gap-2">
          <button
            type="button"
            className="px-3 py-2 rounded-xl bg-white/10 hover:bg-white/20 disabled:opacity-50"
            onClick={handleStopAll}
            disabled={busy !== null}
          >
            {busy === 'stop' ? 'Stopping…' : 'Stop All'}
          </button>
          <button
            type="button"
            className="px-3 py-2 rounded-xl bg-white/10 hover:bg-white/20"
            onClick={() => void refresh()}
          >
            Refresh Laps
          </button>
        </div>
      </section>

      <section className="bg-panel/60 rounded-2xl p-5 shadow-soft border border-white/5">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xl font-semibold">Recent Laps</h2>
          <button
            type="button"
            className="text-sm text-white/60 hover:text-white"
            onClick={() => void refresh()}
          >
            Refresh
          </button>
        </div>

        {laps.length === 0 ? (
          <div className="text-white/60">No laps captured yet.</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {laps.map((l) => (
              <div key={l.id} className="rounded-xl border border-white/5 p-3 bg-white/5">
                <div className="text-sm text-white/60">
                  {l.game} • {l.track} • {l.car}
                </div>
                <div className="text-lg">{fmtLap(l.time_ms)}</div>
                <div className="text-xs text-white/60">Lap #{l.lap_number}</div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
