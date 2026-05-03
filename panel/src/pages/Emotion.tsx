import { useState, useCallback, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Heart,
  Activity,
  Sparkles,
  Brain,
  Gauge,
  Lightbulb,
  Zap,
  Moon,
  Eye,
  RotateCcw,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { api } from '../services/api'

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

interface EmotionData {
  pad: { pleasure: number; arousal: number; dominance: number }
  quadrant: string
  functional_cluster: string
  recommendations: string[]
}

interface PerceptionData {
  total_budget: number
  allocated: number
  available: number
  spent: number
  returned: number
  utilization: number
}

const padLabels: Record<string, { label: string; color: string }> = {
  pleasure: { label: '愉悦度', color: 'var(--accent-cyan)' },
  arousal: { label: '唤醒度', color: 'var(--accent-green)' },
  dominance: { label: '支配度', color: '#a78bfa' },
}

function parseAnchor(input: string): [number, number, number] {
  if (!input.trim()) return [0, 0, 0]
  const parts = input.split(',').map((s) => parseInt(s.trim(), 10))
  if (parts.length === 3 && parts.every((p) => !isNaN(p))) {
    return parts as [number, number, number]
  }
  return [0, 0, 0]
}

export default function Emotion() {
  const [emotion, setEmotion] = useState<EmotionData | null>(null)
  const [perception, setPerception] = useState<PerceptionData | null>(null)
  const [anchorInput, setAnchorInput] = useState('')
  const [pulseLoading, setPulseLoading] = useState(false)
  const [dreamLoading, setDreamLoading] = useState(false)
  const [pulseResult, setPulseResult] = useState<string | null>(null)
  const [dreamResult, setDreamResult] = useState<string | null>(null)

  useEffect(() => {
    api.emotionStatus().then((res) => {
      if (res.success) setEmotion(res.data)
    }).catch(() => {})
    api.perceptionStatus().then((res) => {
      if (res.success) setPerception(res.data)
    }).catch(() => {})
  }, [])

  const handleEmotionPulse = useCallback(async () => {
    setPulseLoading(true)
    setPulseResult(null)
    const anchor = parseAnchor(anchorInput)
    try {
      const res = await api.emotionPulse(anchor)
      if (res.success) {
        setPulseResult(`覆盖 ${res.data.visited_nodes} 节点 · 激活 ${res.data.total_activation.toFixed(2)}`)
        const statusRes = await api.emotionStatus()
        if (statusRes.success) setEmotion(statusRes.data)
      }
    } catch (err: any) {
      setPulseResult(`失败: ${err.message || '未知错误'}`)
    }
    setPulseLoading(false)
  }, [anchorInput])

  const handleEmotionDream = useCallback(async () => {
    setDreamLoading(true)
    setDreamResult(null)
    try {
      const res = await api.emotionDream()
      if (res.success) {
        const d = res.data
        setDreamResult(
          `重播 ${d.paths_replayed} 路径 · 整合 ${d.memories_consolidated} 记忆`
        )
        const statusRes = await api.emotionStatus()
        if (statusRes.success) setEmotion(statusRes.data)
      }
    } catch (err: any) {
      setDreamResult(`失败: ${err.message || '未知错误'}`)
    }
    setDreamLoading(false)
  }, [])

  const handleRefresh = useCallback(async () => {
    api.emotionStatus().then((res) => {
      if (res.success) setEmotion(res.data)
    }).catch(() => {})
    api.perceptionStatus().then((res) => {
      if (res.success) setPerception(res.data)
    }).catch(() => {})
  }, [])

  const padEntries = emotion
    ? [
        { key: 'pleasure', value: emotion.pad.pleasure },
        { key: 'arousal', value: emotion.pad.arousal },
        { key: 'dominance', value: emotion.pad.dominance },
      ]
    : [
        { key: 'pleasure', value: 0 },
        { key: 'arousal', value: 0 },
        { key: 'dominance', value: 0 },
      ]

  return (
    <div className="relative min-h-[100dvh]">
      <div className="relative z-10 p-6">
        <motion.div
          variants={containerVariants}
          initial="hidden"
          animate="visible"
          className="mx-auto max-w-[1440px] space-y-6"
        >
          {/* ─── HEADER ─── */}
          <motion.div variants={cardVariants} className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className="flex h-12 w-12 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-cyan)26' }}
              >
                <Heart className="h-6 w-6" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <div>
                <h1 className="font-display text-2xl font-bold text-[var(--text-primary)]">
                  情绪系统
                </h1>
                <p className="font-body text-[12px] text-[var(--text-muted)]">
                  PAD 情感模型 · 情绪引导交互
                </p>
              </div>
            </div>
            <Button variant="outline" size="sm" onClick={handleRefresh}>
              <RotateCcw className="mr-1.5 h-3.5 w-3.5" />
              刷新
            </Button>
          </motion.div>

          {/* ─── STAT CARDS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {[
              {
                label: '当前象限',
                value: emotion?.quadrant ?? '--',
                icon: Sparkles,
                color: 'var(--accent-cyan)',
              },
              {
                label: '功能聚类',
                value: emotion?.functional_cluster ?? '--',
                icon: Brain,
                color: 'var(--accent-green)',
              },
              {
                label: '愉悦度',
                value: emotion
                  ? `${(emotion.pad.pleasure * 100).toFixed(1)}%`
                  : '--',
                icon: Heart,
                color: 'var(--accent-cyan)',
              },
              {
                label: '唤醒度',
                value: emotion
                  ? `${(emotion.pad.arousal * 100).toFixed(1)}%`
                  : '--',
                icon: Activity,
                color: 'var(--accent-green)',
              },
            ].map((stat) => (
              <motion.div
                key={stat.label}
                variants={cardVariants}
                className="glass-card p-6"
              >
                <div className="mb-3 flex items-center gap-3">
                  <div
                    className="flex h-10 w-10 items-center justify-center rounded-full"
                    style={{ backgroundColor: `${stat.color}26` }}
                  >
                    <stat.icon className="h-5 w-5" style={{ color: stat.color }} />
                  </div>
                  <span className="font-body text-[11px] font-semibold uppercase tracking-[0.06em] text-[var(--text-secondary)]">
                    {stat.label}
                  </span>
                </div>
                <div className="font-mono text-3xl font-bold leading-none tracking-[-0.02em] text-[var(--text-primary)]">
                  {stat.value}
                </div>
              </motion.div>
            ))}
          </div>

          {/* ─── PAD VECTOR DISPLAY ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-cyan)26' }}
              >
                <Gauge className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                PAD 向量
              </h2>
            </div>

            <div className="grid grid-cols-1 gap-6 sm:grid-cols-3">
              {padEntries.map((entry) => {
                const meta = padLabels[entry.key]
                const pct = Math.round(entry.value * 100)
                return (
                  <div
                    key={entry.key}
                    className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4"
                  >
                    <div className="mb-2 flex items-center justify-between">
                      <span className="font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                        {meta.label}
                      </span>
                      <span
                        className="font-mono text-sm font-bold"
                        style={{ color: meta.color }}
                      >
                        {pct}%
                      </span>
                    </div>
                    <div className="h-3 w-full overflow-hidden rounded-full bg-[var(--bg-deep)]">
                      <motion.div
                        initial={{ width: 0 }}
                        animate={{ width: `${pct}%` }}
                        transition={{ duration: 0.8, ease }}
                        className="h-full rounded-full"
                        style={{ backgroundColor: meta.color }}
                      />
                    </div>
                    <div className="mt-3 flex items-center justify-between">
                      <span className="font-mono text-[10px] text-[var(--text-muted)]">
                        0.0
                      </span>
                      <span className="font-mono text-[10px] text-[var(--text-muted)]">
                        1.0
                      </span>
                    </div>
                  </div>
                )
              })}
            </div>

            <div className="mt-4 flex items-center gap-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] p-4">
              <Sparkles className="h-5 w-5 text-[var(--accent-cyan)]" />
              <div className="flex-1">
                <p className="font-body text-[12px] text-[var(--text-muted)]">
                  当前象限
                </p>
                <p className="font-display text-lg font-semibold text-[var(--text-primary)]">
                  {emotion?.quadrant ?? '--'}
                </p>
              </div>
              <div className="h-8 w-px bg-[var(--border-subtle)]" />
              <div className="flex-1">
                <p className="font-body text-[12px] text-[var(--text-muted)]">
                  功能聚类
                </p>
                <Badge
                  variant="outline"
                  className="border-[var(--accent-green)] text-[var(--accent-green)]"
                >
                  {emotion?.functional_cluster ?? '--'}
                </Badge>
              </div>
            </div>
          </motion.div>

          {/* ─── RECOMMENDATIONS ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-green)26' }}
              >
                <Lightbulb className="h-5 w-5" style={{ color: 'var(--accent-green)' }} />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                情绪建议
              </h2>
            </div>

            {emotion?.recommendations && emotion.recommendations.length > 0 ? (
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                {emotion.recommendations.map((rec, i) => (
                  <div
                    key={i}
                    className="flex items-start gap-3 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4"
                  >
                    <div
                      className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[10px] font-bold text-[var(--text-primary)]"
                      style={{ backgroundColor: 'var(--accent-cyan)26' }}
                    >
                      {i + 1}
                    </div>
                    <p className="font-body text-[13px] text-[var(--text-primary)]">
                      {rec}
                    </p>
                  </div>
                ))}
              </div>
            ) : (
              <div className="py-8 text-center">
                <Lightbulb className="mx-auto mb-2 h-8 w-8 text-[var(--text-muted)] opacity-40" />
                <p className="font-body text-[12px] text-[var(--text-muted)]">
                  暂无情绪建议
                </p>
              </div>
            )}
          </motion.div>

          {/* ─── EMOTION-GUIDED ACTIONS ─── */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <div className="mb-4 flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--accent-cyan)26' }}
                >
                  <Zap className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
                </div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  情绪脉冲
                </h2>
              </div>

              <div className="space-y-3">
                <div>
                  <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                    锚点坐标
                  </label>
                  <div className="flex gap-2">
                    <Input
                      placeholder="x,y,z 坐标"
                      value={anchorInput}
                      onChange={(e) => setAnchorInput(e.target.value)}
                      className="font-mono text-xs"
                    />
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() =>
                        setAnchorInput(
                          `${Math.floor(Math.random() * 20)},${Math.floor(Math.random() * 20)},${Math.floor(Math.random() * 20)}`
                        )
                      }
                    >
                      随机
                    </Button>
                  </div>
                </div>
                <Button
                  className="w-full"
                  style={{ backgroundColor: 'var(--accent-cyan)' }}
                  disabled={pulseLoading}
                  onClick={handleEmotionPulse}
                >
                  <Zap className="mr-2 h-4 w-4" />
                  {pulseLoading ? '执行中...' : '触发情绪脉冲'}
                </Button>
                {pulseResult && (
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 py-2">
                    <p className="font-mono text-[11px] text-[var(--text-primary)]">
                      {pulseResult}
                    </p>
                  </div>
                )}
              </div>
            </motion.div>

            <motion.div variants={cardVariants} className="glass-panel p-6">
              <div className="mb-4 flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--accent-green)26' }}
                >
                  <Moon className="h-5 w-5" style={{ color: 'var(--accent-green)' }} />
                </div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  情绪梦境
                </h2>
              </div>

              <div className="flex flex-col items-center justify-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-deep)] p-8">
                <Moon className="mb-3 h-12 w-12 text-[var(--accent-green)] opacity-60" />
                <p className="mb-4 font-body text-sm text-[var(--text-muted)]">
                  执行情绪引导的梦境整合
                </p>
                <Button
                  style={{ backgroundColor: 'var(--accent-green)' }}
                  disabled={dreamLoading}
                  onClick={handleEmotionDream}
                >
                  <Moon className="mr-2 h-4 w-4" />
                  {dreamLoading ? '梦境中...' : '触发情绪梦境'}
                </Button>
              </div>

              {dreamResult && (
                <div className="mt-3 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 py-2">
                  <p className="font-mono text-[11px] text-[var(--text-primary)]">
                    {dreamResult}
                  </p>
                </div>
              )}
            </motion.div>
          </div>

          {/* ─── PERCEPTION BUDGET ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-cyan)26' }}
              >
                <Eye className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                感知预算
              </h2>
            </div>

            {perception ? (
              <div className="space-y-4">
                <div>
                  <div className="mb-2 flex items-center justify-between">
                    <span className="font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                      预算利用率
                    </span>
                    <span className="font-mono text-sm font-bold" style={{ color: 'var(--accent-cyan)' }}>
                      {(perception.utilization * 100).toFixed(1)}%
                    </span>
                  </div>
                  <div className="h-3 w-full overflow-hidden rounded-full bg-[var(--bg-deep)]">
                    <motion.div
                      initial={{ width: 0 }}
                      animate={{ width: `${perception.utilization * 100}%` }}
                      transition={{ duration: 1.2, ease }}
                      className="h-full rounded-full"
                      style={{
                        background:
                          perception.utilization > 0.8
                            ? 'var(--accent-red)'
                            : perception.utilization > 0.5
                              ? 'var(--accent-amber)'
                              : 'var(--accent-green)',
                      }}
                    />
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
                  {[
                    { label: '总预算', value: perception.total_budget },
                    { label: '已分配', value: perception.allocated },
                    { label: '可用', value: perception.available },
                    { label: '已消耗', value: perception.spent },
                    { label: '已回收', value: perception.returned },
                    { label: '利用率', value: perception.utilization, isPercent: true },
                  ].map((item) => (
                    <div
                      key={item.label}
                      className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-3 text-center"
                    >
                      <p className="font-body text-[10px] text-[var(--text-muted)]">
                        {item.label}
                      </p>
                      <p className="font-mono text-lg font-bold text-[var(--text-primary)]">
                        {item.isPercent
                          ? `${(item.value * 100).toFixed(1)}%`
                          : item.value}
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            ) : (
              <div className="py-8 text-center">
                <Eye className="mx-auto mb-2 h-8 w-8 text-[var(--text-muted)] opacity-40" />
                <p className="font-body text-[12px] text-[var(--text-muted)]">
                  加载感知预算...
                </p>
              </div>
            )}
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
