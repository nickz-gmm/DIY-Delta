export default function SectorTable({ sectors }: { sectors: any[] }) {
  return (
    <div>
      <h3 className="font-semibold mb-2">Sectors</h3>
      <table className="w-full text-sm">
        <thead className="text-mute">
          <tr><th className="text-left">#</th><th className="text-left">Start @m</th><th className="text-left">End @m</th></tr>
        </thead>
        <tbody>
          {sectors.map((s:any,i:number)=>(
            <tr key={i} className="border-t border-white/5"><td className="py-1">{i+1}</td><td>{s.start_m.toFixed(1)}</td><td>{s.end_m.toFixed(1)}</td></tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
