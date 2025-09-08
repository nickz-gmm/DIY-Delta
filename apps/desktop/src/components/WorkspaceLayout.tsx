import { useEffect, useMemo, useState } from 'react'

type Widget = {
  id: string
  type: string
  title: string
  content: string
}

type WorkspaceValue = {
  notes?: string
  widgets?: Array<Partial<Widget>>
}

type Props = {
  value?: WorkspaceValue
  onChange: (v: { notes: string; widgets: Widget[] }) => void
}

const uid = () =>
  globalThis.crypto?.randomUUID?.() ??
  Math.random().toString(36).slice(2) + Date.now().toString(36)

function normalizeWidgets(ws: Array<Partial<Widget>> | undefined): Widget[] {
  return (ws ?? []).map((w) => ({
    id: w.id ?? uid(),
    type: w.type ?? 'notes',
    title: w.title ?? 'Notes',
    content: w.content ?? '',
  }))
}

export default function WorkspaceLayout({ value, onChange }: Props) {
  // Initialise from props (with normalization to ensure stable keys)
  const initialNotes = value?.notes ?? ''
  const initialWidgets = useMemo(() => normalizeWidgets(value?.widgets), [value])

  const [notes, setNotes] = useState<string>(initialNotes)
  const [widgets, setWidgets] = useState<Widget[]>(initialWidgets)

  // Keep local state in sync if the parent provides a new `value`
  useEffect(() => {
    setNotes(initialNotes)
  }, [initialNotes])

  useEffect(() => {
    setWidgets(initialWidgets)
  }, [initialWidgets])

  const add = () =>
    setWidgets((w) => [
      ...w,
      { id: uid(), type: 'notes', title: 'Notes', content: '' },
    ])

  const updateWidgetContent = (id: string, content: string) =>
    setWidgets((ws) => ws.map((x) => (x.id === id ? { ...x, content } : x)))

  const save = () => onChange({ notes, widgets })

  return (
    <div className="bg-panel/60 rounded-2xl p-5 border border-white/5 space-y-3">
      <h2 className="text-xl font-semibold">Workspace</h2>

      <textarea
        className="w-full min-h-[120px] bg-white/5 rounded-xl p-3"
        placeholder="Session notes…"
        value={notes}
        onChange={(e) => setNotes(e.target.value)}
      />

      <div className="flex gap-2">
        <button
          type="button"
          className="px-3 py-2 rounded-xl bg-white/10 hover:bg-white/20"
          onClick={add}
        >
          Add Widget
        </button>
        <button
          type="button"
          className="px-3 py-2 rounded-xl bg-accent/20 hover:bg-accent/30"
          onClick={save}
        >
          Save Layout
        </button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
        {widgets.map((w) => (
          <div key={w.id} className="p-3 rounded-xl bg-white/5 border border-white/5">
            <div className="font-semibold mb-2">{w.title}</div>
            <textarea
              className="w-full bg-white/5 rounded-xl p-2"
              placeholder="Widget content…"
              value={w.content}
              onChange={(e) => updateWidgetContent(w.id, e.target.value)}
            />
          </div>
        ))}
      </div>
    </div>
  )
}
