import { useEffect, useState } from 'react'
import { listLaps, buildTrackMap } from '../lib/api'
import TrackMap from '../components/TrackMap'
import SectorTable from '../components/SectorTable'

export default function Track() {
  const [laps, setLaps] = useState<any[]>([])
  const [lapId, setLapId] = useState<string|undefined>()
  const [map, setMap] = useState<any|null>(null)

  useEffect(() => { listLaps().then(setLaps) }, [])

  const run = async () => {
    if(!lapId) return
    setMap(await buildTrackMap(lapId))
  }

  return (
    <div className="space-y-6">
      <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
        <h2 className="text-xl font-semibold mb-2">Build Track Map</h2>
        <div className="flex gap-2">
          <select className="bg-white/5 rounded-xl px-3 py-2" value={lapId} onChange={e=>setLapId(e.target.value)}>
            <option value="">Select lap</option>
            {laps.map(l => <option key={l.id} value={l.id}>{l.game} • {l.track} • Lap {l.lap_number}</option>)}
          </select>
          <button className="px-4 py-2 rounded-xl bg-accent/20 hover:bg-accent/30" onClick={run}>Build</button>
        </div>
      </div>
      {map && (
        <div className="grid grid-cols-3 gap-3">
          <div className="bg-panel/60 rounded-2xl p-5 border border-white/5 col-span-2">
            <TrackMap map={map}/>
          </div>
          <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
            <SectorTable sectors={map.sectors}/>
          </div>
        </div>
      )}
    </div>
  )
}
