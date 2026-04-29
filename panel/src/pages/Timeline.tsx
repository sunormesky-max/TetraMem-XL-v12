import { useState, useEffect, useRef, useCallback } from 'react'
import { motion } from 'framer-motion'
import { Clock, GitBranch, Loader2, ChevronRight, Hash, ExternalLink } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { api } from '../services/api'
import type { TimelineDay, TraceHop } from '../services/api'
import gsap from 'gsap'

const ease = [0.16, 1, 0.3, 1] as [number, number, number, number]

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.06, delayChildren: 0.1 },
  },
}
const cardVariants = {
  hidden: { opacity: 0, y: 24 },
  visible: { opacity: 1, y: 0, transition: { duration: 0.5, ease } },
}

function parseAnchor(anchor: string): [number, number, number] | null {
  const m = anchor.match(/\(\s*(-?[\d.]+)\s*,\s*(-?[\d.]+)\s*,\s*(-?[\d.]+)\s*\)/)
  if (!m) return null
  return [+m[1], +m[2], +m[3]]
}

function formatTime(ms: number): string {
  if (ms <= 0) return '未知'
  const d = new Date(ms)
  return d.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

export default function Timeline() {
  const [timeline, setTimeline] = useState<TimelineDay[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  const [selectedAnchor, setSelectedAnchor] = useState('')
  const [traceAnchor, setTraceAnchor] = useState('100,100,100')
  const [traceResult, setTraceResult] = useState<TraceHop[]>([])
  const [traceLoading, setTraceLoading] = useState(false)
  const [traceError, setTraceError] = useState('')

  const timelineRef = useRef<HTMLDivElement>(null)
  const traceRef = useRef<HTMLDivElement>(null)
  const nodeRefs = useRef<Map<string, HTMLButtonElement>>(new Map())

  const loadTimeline = useCallback(async () => {
    setLoading(true)
    setError('')
    try {
      const res = await api.getTimeline()
      setTimeline(res.data)
    } catch (err: any) {
      setError(err.message || '加载时间轴失败')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadTimeline()
  }, [loadTimeline])

  useEffect(() => {
    if (timeline.length === 0 || !timelineRef.current) return

    const nodes = timelineRef.current.querySelectorAll('.timeline-node')
    const line = timelineRef.current.querySelector('.timeline-line-fill') as HTMLElement

    gsap.fromTo(
      nodes,
      { opacity: 0, x: -30, scale: 0.8 },
      {
        opacity: 1,
        x: 0,
        scale: 1,
        duration: 0.5,
        stagger: 0.08,
        ease: 'power2.out',
      }
    )

    if (line) {
      gsap.fromTo(
        line,
        { scaleY: 0 },
        { scaleY: 1, duration: 0.8, ease: 'power2.out', transformOrigin: 'top center' }
      )
    }
  }, [timeline])

  useEffect(() => {
    if (traceResult.length === 0 || !traceRef.current) return

    const hops = traceRef.current.querySelectorAll('.trace-hop')
    gsap.fromTo(
      hops,
      { opacity: 0, y: 20 },
      {
        opacity: 1,
        y: 0,
        duration: 0.4,
        stagger: 0.1,
        ease: 'power2.out',
      }
    )

    const connectors = traceRef.current.querySelectorAll('.trace-connector')
    connectors.forEach((c) => {
      const el = c as HTMLElement
      gsap.fromTo(el, { scaleX: 0 }, { scaleX: 1, duration: 0.3, ease: 'power2.out' })
    })
  }, [traceResult])

  const handleTrace = async () => {
    const parts = traceAnchor.split(',').map(Number)
    if (parts.length !== 3 || parts.some(isNaN)) {
      setTraceError('锚点坐标必须是3个数字，用逗号分隔')
      return
    }
    setTraceLoading(true)
    setTraceError('')
    setTraceResult([])
    try {
      const res = await api.traceMemory(parts as [number, number, number], 10)
      setTraceResult(res.data)
    } catch (err: any) {
      setTraceError(err.message || '溯源失败')
    } finally {
      setTraceLoading(false)
    }
  }

  const handleNodeClick = (anchor: string) => {
    setSelectedAnchor(selectedAnchor === anchor ? '' : anchor)
    const coords = parseAnchor(anchor)
    if (coords) {
      setTraceAnchor(coords.join(','))
    }
    const el = nodeRefs.current.get(anchor)
    if (el) {
      gsap.to(el, {
        scale: 1.15,
        duration: 0.15,
        yoyo: true,
        repeat: 1,
        ease: 'power2.out',
      })
    }
  }

  const totalMemories = timeline.reduce((s, d) => s + d.count, 0)

  return (
    <div className="relative min-h-[100dvh]">
      <div className="relative z-10 p-6">
        <motion.div
          variants={containerVariants}
          initial="hidden"
          animate="visible"
          className="mx-auto max-w-[1440px] space-y-6"
        >
          {/* Stats Row */}
          <motion.div variants={cardVariants} className="grid grid-cols-3 gap-4">
            <div className="glass-panel flex items-center gap-4 p-4">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--dim-e)26' }}
              >
                <Clock className="h-5 w-5" style={{ color: 'var(--dim-e)' }} />
              </div>
              <div>
                <p className="font-body text-[10px] text-[var(--text-muted)]">活跃天数</p>
                <p className="font-mono text-2xl font-bold text-[var(--text-primary)]">
                  {timeline.length}
                </p>
              </div>
            </div>
            <div className="glass-panel flex items-center gap-4 p-4">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--dim-s)26' }}
              >
                <Hash className="h-5 w-5" style={{ color: 'var(--dim-s)' }} />
              </div>
              <div>
                <p className="font-body text-[10px] text-[var(--text-muted)]">总记忆数</p>
                <p className="font-mono text-2xl font-bold text-[var(--text-primary)]">
                  {totalMemories}
                </p>
              </div>
            </div>
            <div className="glass-panel flex items-center gap-4 p-4">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--dim-t)26' }}
              >
                <GitBranch className="h-5 w-5" style={{ color: 'var(--dim-t)' }} />
              </div>
              <div>
                <p className="font-body text-[10px] text-[var(--text-muted)]">溯源链路</p>
                <p className="font-mono text-2xl font-bold text-[var(--text-primary)]">
                  {traceResult.length}
                </p>
              </div>
            </div>
          </motion.div>

          <div className="grid grid-cols-1 gap-6 lg:grid-cols-5">
            {/* Timeline - left 3 cols */}
            <motion.div
              variants={cardVariants}
              className="glass-panel p-6 lg:col-span-3"
            >
              <div className="mb-4 flex items-center justify-between">
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  记忆时间轴
                </h2>
                <Badge variant="outline" className="font-mono text-xs">
                  {timeline.length} 天
                </Badge>
              </div>

              {loading && (
                <div className="flex items-center justify-center py-12">
                  <Loader2 className="h-6 w-6 animate-spin text-[var(--accent-cyan)]" />
                  <span className="ml-2 font-body text-sm text-[var(--text-muted)]">
                    加载中...
                  </span>
                </div>
              )}

              {error && (
                <div className="mb-4 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                  <span className="font-body text-[12px] text-[var(--accent-red)]">{error}</span>
                </div>
              )}

              {!loading && timeline.length === 0 && (
                <div className="py-12 text-center">
                  <Clock className="mx-auto mb-3 h-10 w-10 text-[var(--text-muted)]" />
                  <p className="font-body text-sm text-[var(--text-muted)]">
                    暂无记忆数据，请先编码记忆
                  </p>
                </div>
              )}

              {!loading && timeline.length > 0 && (
                <div ref={timelineRef} className="relative max-h-[600px] overflow-y-auto pr-2">
                  {/* Vertical line */}
                  <div className="absolute left-[15px] top-0 bottom-0 w-[2px] bg-[var(--border-subtle)]">
                    <div className="timeline-line-fill h-full w-full bg-gradient-to-b from-[var(--accent-cyan)] to-[var(--dim-mu)]" />
                  </div>

                  <div className="space-y-1">
                    {timeline.map((day) => (
                      <div key={day.date} className="timeline-node relative pl-10">
                        {/* Node dot */}
                        <div
                          className="absolute left-[9px] top-[18px] h-[14px] w-[14px] rounded-full border-2 border-[var(--accent-cyan)] bg-[var(--bg-deep)]"
                          style={{
                            boxShadow: '0 0 8px rgba(0,229,255,0.4)',
                          }}
                        />

                        <div className="glass-panel mb-1 p-3">
                          <div className="mb-2 flex items-center justify-between">
                            <span className="font-mono text-sm font-semibold text-[var(--accent-cyan)]">
                              {day.date}
                            </span>
                            <Badge variant="outline" className="font-mono text-[10px]">
                              {day.count} 条
                            </Badge>
                          </div>
                          <div className="flex flex-wrap gap-1.5">
                            {day.anchors.map((a) => (
                              <button
                                key={a}
                                ref={(el) => {
                                  if (el) nodeRefs.current.set(a, el)
                                }}
                                onClick={() => handleNodeClick(a)}
                                className={`inline-flex items-center gap-1 rounded-md px-2 py-1 font-mono text-[10px] transition-all duration-200 ${
                                  selectedAnchor === a
                                    ? 'border border-[var(--accent-cyan)] bg-[var(--accent-cyan)]/20 text-[var(--accent-cyan)]'
                                    : 'border border-[var(--border-subtle)] bg-[var(--bg-surface)] text-[var(--text-secondary)] hover:border-[var(--accent-cyan)]/50 hover:text-[var(--accent-cyan)]'
                                }`}
                              >
                                <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent-cyan)]" />
                                {a.length > 30 ? a.slice(0, 30) + '...' : a}
                              </button>
                            ))}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </motion.div>

            {/* Trace - right 2 cols */}
            <motion.div
              variants={cardVariants}
              className="glass-panel p-6 lg:col-span-2"
            >
              <div className="mb-4 flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--dim-t)26' }}
                >
                  <GitBranch className="h-5 w-5" style={{ color: 'var(--dim-t)' }} />
                </div>
                <div>
                  <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                    记忆溯源
                  </h2>
                  <p className="font-body text-[12px] text-[var(--text-muted)]">
                    追踪关联链路
                  </p>
                </div>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                    起始锚点
                  </label>
                  <Input
                    value={traceAnchor}
                    onChange={(e) => setTraceAnchor(e.target.value)}
                    className="font-mono text-xs"
                    placeholder="x, y, z"
                  />
                </div>

                <Button
                  className="w-full"
                  style={{ backgroundColor: 'var(--dim-t)' }}
                  onClick={handleTrace}
                  disabled={traceLoading}
                >
                  <GitBranch className="mr-2 h-4 w-4" />
                  {traceLoading ? '溯源中...' : '开始溯源'}
                </Button>

                {traceError && (
                  <div className="rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                    <span className="font-body text-[12px] text-[var(--accent-red)]">
                      {traceError}
                    </span>
                  </div>
                )}

                {traceResult.length > 0 && (
                  <div ref={traceRef} className="space-y-0">
                    {traceResult.map((hop, idx) => (
                      <div key={hop.anchor + idx}>
                        <div className="trace-hop flex items-start gap-3 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-3">
                          {/* Hop number */}
                          <div
                            className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full font-mono text-[11px] font-bold"
                            style={{
                              backgroundColor:
                                hop.hop === 0
                                  ? 'var(--accent-cyan)26'
                                  : 'var(--dim-mu)26',
                              color: hop.hop === 0 ? 'var(--accent-cyan)' : 'var(--dim-mu)',
                            }}
                          >
                            {hop.hop}
                          </div>
                          <div className="flex-1 space-y-1">
                            <div className="flex items-center gap-2">
                              <code className="flex-1 truncate font-mono text-[11px] text-[var(--text-primary)]">
                                {hop.anchor}
                              </code>
                            </div>
                            <div className="flex items-center gap-2 text-[10px]">
                              <span className="font-body text-[var(--text-muted)]">
                                {formatTime(hop.created_at)}
                              </span>
                              <Badge variant="outline" className="font-mono text-[9px]">
                                {hop.data_dim}D
                              </Badge>
                              <span
                                className="font-mono font-semibold"
                                style={{
                                  color:
                                    hop.confidence > 0.7
                                      ? 'var(--accent-green)'
                                      : hop.confidence > 0.4
                                      ? 'var(--accent-cyan)'
                                      : 'var(--text-muted)',
                                }}
                              >
                                {(hop.confidence * 100).toFixed(0)}%
                              </span>
                            </div>
                          </div>
                          {hop.hop === 0 && (
                            <Badge
                              className="shrink-0 font-mono text-[9px]"
                              style={{ backgroundColor: 'var(--accent-cyan)', color: '#000' }}
                            >
                              源头
                            </Badge>
                          )}
                        </div>

                        {idx < traceResult.length - 1 && (
                          <div className="flex justify-center py-1">
                            <div className="trace-connector flex h-5 w-[2px] items-center justify-center bg-[var(--dim-t)]/40">
                              <ChevronRight className="h-3 w-3 rotate-90 text-[var(--dim-t)]/60" />
                            </div>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}

                {!traceLoading && traceResult.length === 0 && !traceError && (
                  <div className="py-8 text-center">
                    <ExternalLink className="mx-auto mb-2 h-8 w-8 text-[var(--text-muted)]" />
                    <p className="font-body text-xs text-[var(--text-muted)]">
                      输入锚点坐标并点击溯源，追踪记忆关联链路
                    </p>
                  </div>
                )}
              </div>
            </motion.div>
          </div>
        </motion.div>
      </div>
    </div>
  )
}
