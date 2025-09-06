import { useEffect, useState } from 'react'
import { startF1, startGT7, startLMU, stopAll, listLaps } from '../lib/api'

export default function Dashboard() {
  const [port, setPort] = useState(20777)
  const [format, setFormat] = useState(2025)
  const [consoleIp, setConsoleIp] = useState('192.168.1.100')
  const [variant, setVariant] = useState('A')
  const [laps, setLaps] = useState<any[]>([])

  const refresh = async () => setLaps(await listLaps())

  useEffect(()=>{ refresh() }, [])

  return (
    <div className="space-y-6">
      <section className="bg-panel/60 rounded-2xl p-5 shadow-soft border border-white/5">
        <h2 className="text-xl font-semibold mb-2">Live Sources</h2>
        <div className="grid grid-cols-3 gap-3">
          <div className="p-4 rounded-2xl bg-white/5 border border-white/5 space-y-2">
            <div className="font-semibold">F1 24/25</div>
            <div className="text-sm text-mute">UDP</div>
            <div className="flex gap-2">
              <input className="bg-white/5 rounded-xl px-3 py-2 w-28" type="number" value={port} onChange={e=>setPort(parseInt(e.target.value||'20777'))}/>
              <select className="bg-white/5 rounded-xl px-3 py-2" value={format} onChange={e=>setFormat(parseInt(e.target.value))}>
                <option value={2024}>2024</option>
                <option value={2025}>2025</option>
              </select>
            </div>
            <button className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30" onClick={async()=>{ await startF1(port, format); }}>Start F1</button>
          </div>

          <div className="p-4 rounded-2xl bg-white/5 border border-white/5 space-y-2">
            <div className="font-semibold">Gran Turismo 7</div>
            <div className="text-sm text-mute">PS5 UDP (Salsa20)</div>
            <div className="flex gap-2">
              <input className="bg-white/5 rounded-xl px-3 py-2" value={consoleIp} onChange={e=>setConsoleIp(e.target.value)} placeholder="PS5 IP"/>
              <select className="bg-white/5 rounded-xl px-3 py-2" value={variant} onChange={e=>setVariant(e.target.value)}>
                <option>A</option><option>B</option><option>~</option>
              </select>
            </div>
            <button className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30" onClick={async()=>{ await startGT7(consoleIp, variant); }}>Start GT7</button>
          </div>

          <div className="p-4 rounded-2xl bg-white/5 border border-white/5 space-y-2">
            <div className="font-semibold">Le Mans Ultimate</div>
            <div className="text-sm text-mute">rF2 Shared Memory (Windows)</div>
            <button className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30" onClick={async()=>{ await startLMU(); }}>Start LMU</button>
          </div>
        </div>

        <div className="pt-4">
          <button className="px-3 py-2 rounded-xl bg-white/10 hover:bg-white/20" onClick={async()=>{ await stopAll(); }}>Stop All</button>
          <button className="px-3 py-2 rounded-xl bg-white/10 hover:bg-white/20 ml-2" onClick={refresh}>Refresh Laps</button>
        </div>
      </section>

      <section className="bg-panel/60 rounded-2xl p-5 shadow-soft border border-white/5">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xl font-semibold">Recent Laps</h2>
          <button className="text-sm text-mute hover:text-white" onClick={refresh}>Refresh</button>
        </div>
        {laps.length === 0 ? <div className="text-mute">No laps captured yet.</div> : (
          <div className="grid grid-cols-2 gap-3">
            {laps.map(l => (
              <div key={l.id} className="rounded-xl border border-white/5 p-3 bg-white/5">
                <div className="text-sm text-mute">{l.game} • {l.track} • {l.car}</div>
                <div className="text-lg">{(l.time_ms/1000).toFixed(3)}s</div>
                <div className="text-xs text-mute">Lap #{l.lap_number}</div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
