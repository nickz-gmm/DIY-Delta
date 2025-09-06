export default function TrackMap({ map }: { map: any }) {
  const pad = 20
  const w = 600, h = 600
  const {minx, maxx, miny, maxy} = map.bbox
  const sx = (x:number) => pad + (x - minx) / (maxx - minx) * (w - 2*pad)
  const sy = (y:number) => pad + (1 - (y - miny) / (maxy - miny)) * (h - 2*pad)
  const d = map.polyline.map((p:any,i:number) => `${i?'L':'M'} ${sx(p.x)} ${sy(p.y)}`).join(' ')
  return (
    <svg viewBox={`0 0 ${w} ${h}`} className="w-full h-[500px] bg-white/5 rounded-2xl">
      <path d={d} fill="none" stroke="white" strokeWidth="2"/>
      {map.corners.map((c:any)=> (
        <g key={c.index}>
          <circle cx={sx(c.x)} cy={sy(c.y)} r="4" />
          <text x={sx(c.x)+6} y={sy(c.y)-6} fontSize="10">{c.index}</text>
        </g>
      ))}
    </svg>
  )
}
