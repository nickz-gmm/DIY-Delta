// apps/desktop/src/pages/Track.tsx
import { useCallback, useEffect, useMemo, useState } from 'react'
import TrackMap from '../components/TrackMap'
import SectorTable from '../components/SectorTable'
import { listLaps, getTrackMap } from '../lib/api'

type Lap = {
  id: string
  game: string
  track: string
  car: string
  lap_number: number
  time_ms: number
}

type Point2 = { x: number; y: number }
type BBox = { minx: number; maxx: number; miny: number; maxy: number }
type Sector = { start_m: number; end_m: number }
type CornerLabel = { index: number; x: number; y: number }

type TrackMapData = {
  polyline: Point2[]
  corners: CornerLabel[]
  sectors: Sector[]
  bbox: BBox
}

export default function Track() {
  const [laps, setLaps] = useState<Lap[]>([])
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [map, setMap] = useState<TrackMapData | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadLaps = useCallback(async () => {
    try {
      setError(null)
      const res = await listLaps()
      const list = Array.isArray(res) ? (res as Lap[]) : []
      setLaps(list)
    } catch (e: any) {
      setError(e?.message ?? 'Failed to load laps')
      setLaps([])
    }
  }, [])

  // Initial fetch
  useEffect(() => {
    void loadLaps()
  }, [loadLaps])

  // Keep selection stable after refresh; pick first if none selected
  useEffect(() => {
    if (!laps.length) {
      setSelectedId(null)
      return
    }
    // keep current if still present, else choose first
    if (!selectedId || !laps.some((l) => l.id === selectedId)) {
      setSelectedId(laps[0].id)
    }
  }, [laps, selectedId])

  const loadMap = useCallback(async (lapId: string) => {
    try {
      setError(null)
      setLoading(true)
      const res = (await getTrackMap(lapId)) as TrackMapData
      setMap(res ?? null)
    } catch (e: any) {
      setError(e?.message ?? 'Failed to load track map')
      setMap(null)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    if (selectedId) void loadMap(selectedId)
  }, [selectedId, loadMap])

  const selectedLap = useMemo(
    () => laps.find((l) => l.id === selectedId) || null,
    [laps, selectedId]
  )

  const fmtLap = (ms?: number) =>
    Number.isFinite(ms) ? `${(ms! / 1000).toFixed(3)}s` : '-'

  return (
    <div className="space-y-6">
      <section className="bg-panel/60 rounded-2xl p-5 border border-white/5">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <h2 className="text-xl font-semibold">Track</h2>
          <div className="flex items-center gap-2">
            <select
              className="bg-white/5 rounded-xl px-3 py-2 min-w-[220px]"
              value={selectedId ?? ''}
              onChange={(e) => setSelectedId(e.target.value || null)}
            >
              {laps.length === 0 && <option value="">No laps available</option>}
              {laps.map((l) => (
                <option key={l.id} value={l.id}>
                  {l.game} • {l.track} • {l.car} • Lap {l.lap_number} • {fmtLap(l.time_ms)}
                </option>
              ))}
            </select>
            <button
              type="button"
              className="px-3 py-2 rounded-xl bg-white/10 hover:bg-white/20"
              onClick={() => void loadLaps()}
            >
              Refresh
            </button>
          </div>
        </div>

        {error && <div className="mt-3 text-sm text-red-400">{error}</div>}
      </section>

      <section className="bg-panel/60 rounded-2xl p-5 border border-white/5">
        <h3 className="font-semibold mb-3">
          {selectedLap
            ? `${selectedLap.track} • ${selectedLap.car} (Lap ${selectedLap.lap_number})`
            : 'No lap selected'}
        </h3>

        {loading ? (
          <div className="h-[500px] grid place-items-center text-white/70">
            Loading track map…
          </div>
        ) : map ? (
          <TrackMap map={map} />
        ) : (
          <div className="h-[200px] grid place-items-center text-white/60">
            No track map available.
          </div>
        )}
      </section>

      <section className="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
          <SectorTable sectors={map?.sectors ?? []} />
        </div>

        <div className="lg:col-span-2 bg-panel/60 rounded-2xl p-5 border border-white/5">
          <h3 className="font-semibold mb-2">Corner Labels</h3>
          {map?.corners?.length ? (
            <table className="w-full text-sm">
              <thead className="text-white/70">
                <tr>
                  <th className="text-left py-1">#</th>
                  <th className="text-left">X</th>
                  <th className="text-left">Y</th>
                  <th className="text-left">Start (m)</th>
                  <th className="text-left">End (m)</th>
                </tr>
              </thead>
              <tbody>
                {map.corners.map((c) => {
                  // Best-effort: pair corner index (1-based) to sector at same index
                  const sec = map.sectors?.[Math.max(0, c.index - 1)]
                  return (
                    <tr key={c.index} className="border-t border-white/5">
                      <td className="py-1">{c.index}</td>
                      <td>{Number.isFinite(c.x) ? c.x.toFixed(2) : '-'}</td>
                      <td>{Number.isFinite(c.y) ? c.y.toFixed(2) : '-'}</td>
                      <td>
                        {sec && Number.isFinite(sec.start_m)
                          ? sec.start_m.toFixed(1)
                          : '-'}
                      </td>
                      <td>
                        {sec && Number.isFinite(sec.end_m)
                          ? sec.end_m.toFixed(1)
                          : '-'}
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          ) : (
            <div className="text-sm text-white/60">No corner labels found.</div>
          )}
        </div>
      </section>
    </div>
  )
}
