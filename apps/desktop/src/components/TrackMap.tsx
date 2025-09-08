import { useMemo } from 'react'

type Point = { x: number; y: number }
type BBox = { minx: number; maxx: number; miny: number; maxy: number }
type Corner = { index: number; x: number; y: number; label?: string }

type TrackMapData = {
  bbox: BBox
  polyline: Point[]
  corners?: Corner[]
}

type Props = {
  map?: TrackMapData | null
  width?: number
  height?: number
  padding?: number
  className?: string
}

function isFiniteNumber(n: unknown): n is number {
  return typeof n === 'number' && Number.isFinite(n)
}

export default function TrackMap({
  map,
  width = 600,
  height = 600,
  padding = 20,
  className = 'w-full h-[500px] bg-white/5 rounded-2xl text-white',
}: Props) {
  // Basic validation
  const valid =
    map &&
    map.bbox &&
    ['minx', 'maxx', 'miny', 'maxy'].every((k) =>
      isFiniteNumber((map.bbox as any)[k])
    ) &&
    Array.isArray(map.polyline) &&
    map.polyline.length > 0

  if (!valid) {
    return (
      <div className={`${className} grid place-items-center`}>
        <span className="text-sm text-white/70">No track map available.</span>
      </div>
    )
  }

  const {
    bbox: { minx, maxx, miny, maxy },
    polyline,
    corners = [],
  } = map as TrackMapData

  // Prevent divide-by-zero by enforcing a minimal span
  const spanX = Math.max(1e-6, maxx - minx)
  const spanY = Math.max(1e-6, maxy - miny)

  const innerW = Math.max(0, width - 2 * padding)
  const innerH = Math.max(0, height - 2 * padding)

  const sx = (x: number) => padding + ((x - minx) / spanX) * innerW
  // Flip Y so higher Y plots visually "up"
  const sy = (y: number) => padding + (1 - (y - miny) / spanY) * innerH

  const pathD = useMemo(() => {
    return polyline
      .map((p, i) => {
        const px = isFiniteNumber(p.x) ? p.x : 0
        const py = isFiniteNumber(p.y) ? p.y : 0
        return `${i ? 'L' : 'M'} ${sx(px)} ${sy(py)}`
      })
      .join(' ')
  }, [polyline, minx, maxx, miny, maxy, padding, width, height])

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      className={className}
      role="img"
      aria-label="Track map"
    >
      <g fill="none" stroke="currentColor" strokeWidth={2} vectorEffect="non-scaling-stroke">
        <path d={pathD} />
      </g>

      {/* Corners */}
      <g>
        {corners.map((c) => {
          const cx = sx(isFiniteNumber(c.x) ? c.x : 0)
          const cy = sy(isFiniteNumber(c.y) ? c.y : 0)
          const label = isFiniteNumber(c.index) ? String(c.index) : c.label ?? ''
          return (
            <g key={label ? `corner-${label}` : `corner-${cx}-${cy}`}>
              <circle
                cx={cx}
                cy={cy}
                r={4}
                fill="currentColor"
                opacity={0.9}
              />
              {label && (
                <text
                  x={cx + 6}
                  y={cy - 6}
                  fontSize={10}
                  className="fill-current"
                >
                  {label}
                </text>
              )}
            </g>
          )
        })}
      </g>
    </svg>
  )
}
