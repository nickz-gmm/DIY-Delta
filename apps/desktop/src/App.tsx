import { Link, Outlet, useLocation } from 'react-router-dom'
import { NotebookPen, Route, Timer, Gauge } from 'lucide-react'

export default function App() {
  const location = useLocation()
  const tabs = [
    { to: '/', label: 'Dashboard', icon: <Gauge size={18}/> },
    { to: '/laps', label: 'Laps', icon: <Timer size={18}/> },
    { to: '/track', label: 'Track', icon: <Route size={18}/> },
    { to: '/workspace', label: 'Workspace', icon: <NotebookPen size={18}/> },
  ]
  return (
    <div className="h-full flex">
      <aside className="w-56 bg-panel/60 border-r border-white/5 p-4">
        <div className="text-2xl font-semibold tracking-tight mb-6">Delta</div>
        <nav className="space-y-2">
          {tabs.map(t => (
            <Link key={t.to} to={t.to}
              className={"flex items-center gap-2 px-3 py-2 rounded-xl hover:bg-white/5 transition " + (location.pathname===t.to ? "bg-white/5" : "")}>
                {t.icon}<span>{t.label}</span>
            </Link>
          ))}
        </nav>
        <div className="mt-8 text-sm text-mute">v0.2 • Minimalist • Cross-platform</div>
      </aside>
      <main className="flex-1 p-6 overflow-auto">
        <Outlet/>
      </main>
    </div>
  )
}
