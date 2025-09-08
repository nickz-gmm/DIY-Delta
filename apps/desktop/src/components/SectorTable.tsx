type Sector = {
  start_m: number
  end_m: number
}

type Props = {
  sectors?: Sector[] | null
}

export default function SectorTable({ sectors }: Props) {
  const safeSectors: Sector[] = Array.isArray(sectors) ? sectors : []

  if (!safeSectors.length) {
    return (
      <div className="text-sm text-muted italic">
        No sectors available.
      </div>
    )
  }

  return (
    <div>
      <h3 className="font-semibold mb-2">Sectors</h3>
      <table className="w-full text-sm border-collapse">
        <thead className="text-muted">
          <tr>
            <th className="text-left">#</th>
            <th className="text-left">Start&nbsp;(m)</th>
            <th className="text-left">End&nbsp;(m)</th>
          </tr>
        </thead>
        <tbody>
          {safeSectors.map((s, i) => (
            <tr key={i} className="border-t border-white/10">
              <td className="py-1">{i + 1}</td>
              <td>{Number.isFinite(s.start_m) ? s.start_m.toFixed(1) : '-'}</td>
              <td>{Number.isFinite(s.end_m) ? s.end_m.toFixed(1) : '-'}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
