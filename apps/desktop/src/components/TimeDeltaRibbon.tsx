import { useMemo } from 'react'
import {
  Area,
  AreaChart,
  Tooltip,
  XAxis,
  YAxis,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts'

type Point = {
  distance: number
  delta_ms: number
}

type Props = {
  data?: Point[] | null
}

function isFiniteNumber(n: unknown): n is number {
  return typeof n === 'number' && Number.isFinite(n)
}

const fmtNum = (v?: number) =>
  isFiniteNumber(v) ? (Math.abs(v) >= 100 ? v.toFixed(0) : v.toFixed(2)) : ''

const fmtMs = (v?: number) =>
  isFiniteNumber(v) ? `${v >= 0 ? '+' : ''}${fmtNum(v)} ms` : ''

export default function TimeDeltaRibbon({ data }: Props) {
  const safeData: Point[] = Array.isArray(data) ? data : []

  const yDomain = useMemo<[number, number]>(() => {
    if (!safeData.length) return [-1, 1]
    const maxAbs = safeData.reduce((m, p) => {
      const v = isFiniteNumber(p.delta_ms) ? Math.abs(p.delta_ms) : 0
      return Math.max(m, v)
    }, 0)
    const pad = maxAbs * 0.1
    const span = Math.max(1, maxAbs + pad)
    return [-span, span]
  }, [safeData])

  if (!safeData.length) {
    return (
      <div className="h-48 grid place-items-center rounded-xl border border-white/10 bg-white/5 text-sm text-white/70">
        No delta data to display.
      </div>
    )
  }

  return (
    <div className="h-48">
      <ResponsiveContainer>
        <AreaChart
          data={safeData}
          margin={{ top: 8, right: 16, bottom: 8, left: 8 }}
        >
          <defs>
            <linearGradient id="deltaFill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#3b82f6" stopOpacity={0.35} />
              <stop offset="100%" stopColor="#3b82f6" stopOpacity={0.05} />
            </linearGradient>
          </defs>

          <XAxis
            dataKey="distance"
            type="number"
            tickFormatter={(v) => `${fmtNum(v)}`}
            label={{ value: 'Distance (m)', position: 'insideBottomRight', offset: -4 }}
            allowDecimals
          />
          <YAxis
            domain={yDomain}
            tickFormatter={(v) => fmtNum(v)}
            width={48}
            allowDecimals
          />
          <Tooltip
            formatter={(val: any) => fmtMs(val)}
            labelFormatter={(label: any) =>
              isFiniteNumber(label) ? `${fmtNum(label)} m` : String(label ?? '')
            }
          />

          <ReferenceLine y={0} stroke="#94a3b8" strokeDasharray="4 4" />

          <Area
            type="monotone"
            dataKey="delta_ms"
            stroke="#3b82f6"
            strokeWidth={2}
            fill="url(#deltaFill)"
            dot={false}
            connectNulls
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  )
}
