import { useState } from 'react'
export default function WorkspaceLayout({ value, onChange }:{ value:any, onChange:(v:any)=>void }) {
  const [notes, setNotes] = useState(value?.notes || '')
  const [widgets, setWidgets] = useState<any[]>(value?.widgets || [])

  const add = () => setWidgets(w => [...w, { type: 'notes', title: 'Notes', content: '' }])
  const save = () => onChange({ notes, widgets })

  return (
    <div className="bg-panel/60 rounded-2xl p-5 border border-white/5 space-y-3">
      <h2 className="text-xl font-semibold">Workspace</h2>
      <textarea className="w-full min-h-[120px] bg-white/5 rounded-xl p-3" placeholder="Session notes…"
        value={notes} onChange={e=>setNotes(e.target.value)} />
      <div className="flex gap-2">
        <button className="px-3 py-2 rounded-xl bg-white/10 hover:bg-white/20" onClick={add}>Add Widget</button>
        <button className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30" onClick={save}>Save Layout</button>
      </div>
      <div className="grid grid-cols-2 gap-2">
        {widgets.map((w,i) => (
          <div key={i} className="p-3 rounded-xl bg-white/5 border border-white/5">
            <div className="font-semibold mb-2">{w.title}</div>
            <textarea className="w-full bg-white/5 rounded-xl p-2" placeholder="Widget content…"
              value={w.content} onChange={e=>setWidgets(ws => ws.map((x,j)=> j===i ? {...x, content:e.target.value} : x))}/>
          </div>
        ))}
      </div>
    </div>
  )
}
