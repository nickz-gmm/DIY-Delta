import { useEffect, useState } from 'react'
import { analyzeLaps, listLaps } from '../lib/api'
import LapOverlayChart from '../components/LapOverlayChart'
import TimeDeltaRibbon from '../components/TimeDeltaRibbon'

export default function Laps() {
  const [laps, setLaps] = useState<any[]>([])
  const [selected, setSelected] = useState<string[]>([])
  const [analysis, setAnalysis] = useState<any|null>(null)

  useEffect(() => { listLaps().then(setLaps) }, [])

  const run = async () => {
    if(selected.length<1) return
    const res = await analyzeLaps(selected)
    setAnalysis(res)
  }

  return (
    <div className="space-y-6">
      <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
        <h2 className="text-xl font-semibold mb-3">Select Laps</h2>
        <div className="grid grid-cols-3 gap-2 mb-4">
          {laps.map(l => (
            <label key={l.id} className={"p-3 rounded-xl border " + (selected.includes(l.id) ? "border-accent bg-accent/10" : "border-white/5 bg-white/5")}>
              <input type="checkbox" className="mr-2" checked={selected.includes(l.id)} onChange={e=>{
                setSelected(x => e.target.checked ? [...x, l.id] : x.filter(z=>z!==l.id))
              }}/>
              <span className="text-sm">{l.game} • {l.track} • {l.car} • Lap {l.lap_number} • {(l.time_ms/1000).toFixed(3)}s</span>
            </label>
          ))}
        </div>
        <button className="px-4 py-2 rounded-xl bg-accent/20 hover:bg-accent/30" onClick={run}>Analyze</button>
      </div>

      {analysis && (
        <div className="space-y-6">
          <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
            <h3 className="font-semibold mb-2">Overlay: Speed vs Lap Distance</h3>
            <LapOverlayChart data={analysis.overlay}/>
          </div>
          <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
            <h3 className="font-semibold mb-2">Time Delta Ribbon (vs Reference)</h3>
            <TimeDeltaRibbon data={analysis.delta_ribbon} />
          </div>
          <div className="grid grid-cols-3 gap-3">
            <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
              <h4 className="font-semibold mb-2">Summary</h4>
              <ul className="text-sm space-y-1 text-mute">
                <li>Best lap: {(analysis.summary.best_ms/1000).toFixed(3)}s</li>
                <li>Worst lap: {(analysis.summary.worst_ms/1000).toFixed(3)}s</li>
                <li>Average lap: {(analysis.summary.avg_ms/1000).toFixed(3)}s</li>
                <li>Consistency (sector σ): {analysis.summary.consistency.toFixed(3)}s</li>
              </ul>
            </div>
            <div className="bg-panel/60 rounded-2xl p-5 border border-white/5 col-span-2">
              <h4 className="font-semibold mb-2">Per-Corner Metrics (Reference)</h4>
              <table className="w-full text-sm">
                <thead className="text-mute">
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
                  {analysis.corners.map((c:any)=> (
                    <tr key={c.index} className="border-t border-white/5">
                      <td className="py-1">{c.index}</td>
                      <td>{c.min_speed.toFixed(1)} km/h</td>
                      <td>{c.entry_speed.toFixed(1)}</td>
                      <td>{c.exit_speed.toFixed(1)}</td>
                      <td>{c.brake_point_m.toFixed(1)}</td>
                      <td>{c.throttle_on_m.toFixed(1)}</td>
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
