import { Line, LineChart, Tooltip, XAxis, YAxis, Legend, ResponsiveContainer } from 'recharts'
export default function LapOverlayChart({ data }: { data: any[] }) {
  const keys = Object.keys(data?.[0]||{}).filter(k=>k!=='distance')
  return (
    <div className="h-80">
      <ResponsiveContainer>
        <LineChart data={data}>
          <XAxis dataKey="distance" />
          <YAxis />
          <Tooltip />
          <Legend />
          {keys.map(k => <Line key={k} type="monotone" dataKey={k} dot={false} />)}
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
