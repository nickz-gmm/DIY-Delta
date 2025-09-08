import { useCallback, useEffect, useMemo, useState } from 'react'
import { analyzeLaps, listLaps } from '../lib/api'
import LapOverlayChart from '../components/LapOverlayChart'
import TimeDeltaRibbon from '../components/TimeDeltaRibbon'

type Lap = {
  id: string
  game: string
  track: string
  car: string
  lap_number: number
  time_ms: number
}

type OverlayPoint = {
  distance: number
  [series: string]: number // e.g., speed, throttle, etc.
}

type DeltaPoint = {
  distance: number
  delta_ms: number
}

type CornerMetrics = {
  index: number
  min_speed: number
  entry_speed: number
  exit_speed: number
  brake_point_m: number
  throttle_on_m: number
}

type Analysis = {
  overlay: OverlayPoint[]
  delta_ribbon: DeltaPoint[]
  summary: {
    best_ms: number
    worst_ms: number
    avg_ms: number
    consistency: number
  }
  corners: CornerMetrics[]
}

export default function Laps() {
  const [laps, setLaps] = useState<Lap[]>([])
  const [selected, setSelected] = useState<string[]>([])
  const [analysis, setAnalysis] = useState<Analysis | null>(null)
  const [busy, setBusy] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)

  const selectedSet = useMemo(() => new Set(selected), [selected])

  const fmtLap = (ms?: number) =>
    Number.isFinite(ms) ? `${(ms! / 1000).toFixed(3)}s` : '-'

  const loadLaps = useCallback(async () => {
    try {
      setError(null)
      const res = await listLaps()
      setLaps(Array.isArray(res) ? (res as Lap[]) : [])
    } catch (e: any) {
      setError(e?.message ?? 'Failed to load laps')
      setLaps([])
    }
  }, [])

  useEffect(() => {
    void loadLaps()
  }, [loadLaps])

  const toggle = (id: string, checked: boolean) => {
    setSelected((prev) => {
      if (checked) {
        if (prev.includes(id)) return prev
        return [...prev, id]
      }
      return prev.filter((x) => x !== id)
    })
  }

  const run = useCallback(async () => {
    if (selected.length < 1 || busy) return
    try {
      setBusy(true)
      setError(null)
      const res = (await analyzeLaps(selected)) as Analysis
      // quick sanity guard
      setAnalysis(res && typeof res === 'object' ? res : null)
    } catch (e: any) {
      setError(e?.message ?? 'Analysis failed')
      setAnalysis(null)
    } finally {
      setBusy(false)
    }
  }, [selected, busy])

  return (
    <div className="space-y-6">
      <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
        <h2 className="text-xl font-semibold mb-3">Select Laps</h2>

        {error && (
          <div className="mb-3 text-sm text-red-400">{error}</div>
        )}

        <div className="grid grid-cols-1 md:grid-cols-3 gap-2 mb-4">
          {laps.map((l) => {
            const id = `lap-${l.id}`
            const isChecked = selectedSet.has(l.id)
            return (
              <label
                key={l.id}
                htmlFor={id}
                className={
                  'p-3 rounded-xl border cursor-pointer select-none ' +
                  (isChecked
                    ? 'border-accent bg-accent/10'
                    : 'border-white/5 bg-white/5')
                }
              >
                <input
                  id={id}
                  type="checkbox"
                  className="mr-2 align-middle"
                  checked={isChecked}
                  onChange={(e) => toggle(l.id, e.target.checked)}
                />
                <span className="text-sm">
                  {l.game} • {l.track} • {l.car} • Lap {l.lap_number} •{' '}
                  {fmtLap(l.time_ms)}
                </span>
              </label>
            )
          })}
        </div>

        <button
          className="px-4 py-2 rounded-xl bg-accent/20 hover:bg-accent/30 disabled:opacity-50"
          onClick={() => void run()}
          disabled={busy || selected.length < 1}
        >
          {busy ? 'Analyzing…' : 'Analyze'}
        </button>
      </div>

      {analysis && (
        <div className="space-y-6">
          <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
            <h3 className="font-semibold mb-2">Overlay: Speed vs Lap Distance</h3>
            <LapOverlayChart data={analysis.overlay ?? []} />
          </div>

          <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
            <h3 className="font-semibold mb-2">Time Delta Ribbon (vs Reference)</h3>
            <TimeDeltaRibbon data={analysis.delta_ribbon ?? []} />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
            <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
              <h4 className="font-semibold mb-2">Summary</h4>
              <ul className="text-sm space-y-1 text-white/70">
                <li>Best lap: {fmtLap(analysis.summary?.best_ms)}</li>
                <li>Worst lap: {fmtLap(analysis.summary?.worst_ms)}</li>
                <li>Average lap: {fmtLap(analysis.summary?.avg_ms)}</li>
                <li>
                  Consistency (sector σ):{' '}
                  {Number.isFinite(analysis.summary?.consistency)
                    ? analysis.summary.consistency.toFixed(3) + 's'
                    : '-'}
                </li>
              </ul>
            </div>

            <div className="bg-panel/60 rounded-2xl p-5 border border-white/5 lg:col-span-2">
              <h4 className="font-semibold mb-2">Per-Corner Metrics (Reference)</h4>
              <table className="w-full text-sm">
                <thead className="text-white/70">
                  <tr>
                    <th className="text-left py-1">#</th>
                    <th className="text-left">Min Speed</th>
                    <th className="text-left">Entry</th>
                    <th className="text-left">Exit</th>
                    <th className="text-left">Brake On @m</th>
                    <th className="text-left">Throttle @m</th>
                  </tr>
                </thead>
                <tbody>
                  {(analysis.corners ?? []).map((c) => (
                    <tr key={c.index} className="border-t border-white/5">
                      <td className="py-1">{c.index}</td>
                      <td>
                        {Number.isFinite(c.min_speed)
                          ? `${c.min_speed.toFixed(1)} km/h`
                          : '-'}
                      </td>
                      <td>
                        {Number.isFinite(c.entry_speed)
                          ? c.entry_speed.toFixed(1)
                          : '-'}
                      </td>
                      <td>
                        {Number.isFinite(c.exit_speed)
                          ? c.exit_speed.toFixed(1)
                          : '-'}
                      </td>
                      <td>
                        {Number.isFinite(c.brake_point_m)
                          ? c.brake_point_m.toFixed(1)
                          : '-'}
                      </td>
                      <td>
                        {Number.isFinite(c.throttle_on_m)
                          ? c.throttle_on_m.toFixed(1)
                          : '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
