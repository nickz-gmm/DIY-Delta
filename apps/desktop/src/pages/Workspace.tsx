import { useEffect, useState } from 'react'
import { listWorkspaces, saveWorkspace, loadWorkspace } from '../lib/api'
import WorkspaceLayout from '../components/WorkspaceLayout'

export default function Workspace() {
  const [name, setName] = useState('Default')
  const [all, setAll] = useState<string[]>([])
  const [layout, setLayout] = useState<any>({ notes: '', widgets: [] })

  const refresh = async () => setAll(await listWorkspaces())

  useEffect(()=>{ refresh(); (async()=>{
    const data = await loadWorkspace(name).catch(()=>null)
    if(data) setLayout(data)
  })() }, [])

  return (
    <div className="space-y-6">
      <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
        <div className="flex items-center gap-2">
          <input className="bg-white/5 rounded-xl px-3 py-2" value={name} onChange={e=>setName(e.target.value)}/>
          <button className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30" onClick={async ()=>{ await saveWorkspace(name, layout); await refresh() }}>Save</button>
          <select className="bg-white/5 rounded-xl px-3 py-2" onChange={async e=>{ setName(e.target.value); const d = await loadWorkspace(e.target.value); setLayout(d) }} value={name}>
            <option value="Default">Default</option>
            {all.filter(x=>x!=='Default').map(n => <option key={n} value={n}>{n}</option>)}
          </select>
        </div>
      </div>
      <WorkspaceLayout value={layout} onChange={setLayout}/>
    </div>
  )
}
