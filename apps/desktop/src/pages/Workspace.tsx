import { useCallback, useEffect, useState } from 'react'
import { listWorkspaces, saveWorkspace, loadWorkspace } from '../lib/api'
import WorkspaceLayout from '../components/WorkspaceLayout'

type WorkspaceValue = { notes: string; widgets: any[] }

export default function Workspace() {
  const [name, setName] = useState('Default')
  const [all, setAll] = useState<string[]>([])
  const [layout, setLayout] = useState<WorkspaceValue>({ notes: '', widgets: [] })
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    try {
      setError(null)
      const names = await listWorkspaces()
      setAll(Array.isArray(names) ? names : [])
    } catch (e: any) {
      setError(e?.message ?? 'Failed to list workspaces')
      setAll([])
    }
  }, [])

  // Initial fetch of workspace names
  useEffect(() => {
    void refresh()
  }, [refresh])

  // Load the selected workspace whenever `name` changes
  useEffect(() => {
    let cancelled = false
    ;(async () => {
      try {
        setBusy(true)
        setError(null)
        const data = await loadWorkspace(name)
        if (!cancelled && data) {
          setLayout({
            notes: data.notes ?? '',
            widgets: Array.isArray(data.widgets) ? data.widgets : [],
          })
        }
      } catch (e: any) {
        if (!cancelled) setError(e?.message ?? 'Failed to load workspace')
      } finally {
        if (!cancelled) setBusy(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [name])

  const onSave = useCallback(async () => {
    const trimmed = name.trim() || 'Default'
    try {
      setBusy(true)
      setError(null)
      await saveWorkspace(trimmed, layout)
      if (trimmed !== name) setName(trimmed) // normalise after save
      await refresh()
    } catch (e: any) {
      setError(e?.message ?? 'Failed to save workspace')
    } finally {
      setBusy(false)
    }
  }, [name, layout, refresh])

  const hasDefault = all.includes('Default')
  const namesForSelect = hasDefault ? all : ['Default', ...all]

  return (
    <div className="space-y-6">
      <div className="bg-panel/60 rounded-2xl p-5 border border-white/5">
        <div className="flex items-center gap-2">
          <input
            className="bg-white/5 rounded-xl px-3 py-2"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Workspace name"
          />
          <button
            className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30 disabled:opacity-50"
            onClick={() => void onSave()}
            disabled={busy || !name.trim()}
          >
            {busy ? 'Saving…' : 'Save'}
          </button>
          <select
            className="bg-white/5 rounded-xl px-3 py-2"
            value={name}
            onChange={(e) => setName(e.target.value || 'Default')}
          >
            {namesForSelect.length === 0 && <option value="Default">Default</option>}
            {namesForSelect.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
          <button
            className="px-3 py-2 rounded-xl bg-white/10 hover:bg-white/20"
            onClick={() => void refresh()}
          >
            Refresh
          </button>
        </div>
        {error && <div className="mt-3 text-sm text-red-400">{error}</div>}
      </div>

      <WorkspaceLayout value={layout} onChange={setLayout} />
    </div>
  )
}
