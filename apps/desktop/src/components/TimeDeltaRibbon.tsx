import { Area, AreaChart, Tooltip, XAxis, YAxis, ResponsiveContainer } from 'recharts'
export default function TimeDeltaRibbon({ data }: { data: any[] }) {
  return (
    <div className="h-48">
      <ResponsiveContainer>
        <AreaChart data={data}>
          <XAxis dataKey="distance" />
          <YAxis />
          <Tooltip />
          <Area type="monotone" dataKey="delta_ms" />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  )
}
