import { useMemo } from 'react'
import {
  Line,
  LineChart,
  Tooltip,
  XAxis,
  YAxis,
  Legend,
  ResponsiveContainer,
} from 'recharts'

type Point = {
  distance: number
  speed: number
  throttle: number
  brake: number
  // additional numeric series keyed by name: speed, throttle, brake, etc.
  [series: string]: number
}

type Props = {
  data?: Point[] | null
}

function isFiniteNumber(n: unknown): n is number {
  return typeof n === 'number' && Number.isFinite(n)
}

const palette = [
  '#3b82f6', // blue-500
  '#10b981', // emerald-500
  '#f59e0b', // amber-500
  '#ef4444', // red-500
  '#8b5cf6', // violet-500
  '#06b6d4', // cyan-500
  '#e11d48', // rose-600
  '#84cc16', // lime-500
  '#0ea5e9', // sky-500
  '#a855f7', // purple-500
]

const fmtNum = (v?: number) =>
  isFiniteNumber(v) ? (Math.abs(v) >= 100 ? v.toFixed(0) : v.toFixed(2)) : ''

export default function LapOverlayChart({ data }: Props) {
  const safeData: Point[] = Array.isArray(data) ? (data as Point[]) : []

  // Determine which keys are valid numeric series (exclude "distance")
  const seriesKeys = useMemo(() => {
    if (safeData.length === 0) return [] as string[]
    const candidateKeys = Object.keys(safeData[0] ?? {}).filter(
      (k) => k !== 'distance'
    )
    // keep keys whose values look numeric for at least one point
    return candidateKeys.filter((k) =>
      safeData.some((p) => isFiniteNumber((p as any)[k]))
    )
  }, [safeData])

  if (!safeData.length) {
    return (
      <div className="h-80 grid place-items-center rounded-xl border border-white/10 bg-white/5 text-sm text-white/70">
        No lap data to display.
      </div>
    )
  }

  return (
    <div className="h-80">
      <ResponsiveContainer>
        <LineChart
          data={safeData}
          margin={{ top: 8, right: 16, bottom: 8, left: 8 }}
        >
          <XAxis
            dataKey="distance"
            tickFormatter={fmtNum}
            label={{ value: 'Distance (m)', position: 'insideBottomRight', offset: -4 }}
            type="number"
            allowDecimals
          />
          <YAxis
            tickFormatter={fmtNum}
            allowDecimals
            width={48}
          />
          <Tooltip
            formatter={(val: any) =>
              isFiniteNumber(val) ? fmtNum(val) : String(val ?? '')
            }
            labelFormatter={(label: any) =>
              isFiniteNumber(label) ? `${fmtNum(label)} m` : String(label ?? '')
            }
          />
          <Legend />
          {seriesKeys.map((k, idx) => (
            <Line
              key={k}
              type="monotone"
              dataKey={k}
              dot={false}
              isAnimationActive={false}
              stroke={palette[idx % palette.length]}
              strokeWidth={2}
              connectNulls
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
