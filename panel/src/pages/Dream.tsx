import { useState, useCallback, useEffect, useRef } from 'react'
import { motion } from 'framer-motion'
import {
  Moon,
  Activity,
  TrendingDown,
  TrendingUp,
  RotateCcw,
  Clock,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Slider } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import { Badge } from '@/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
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

type DreamStatus = 'completed' | 'running' | 'failed'
type DreamPhase = 'light' | 'medium' | 'deep'

const phaseLabels: Record<DreamPhase, string> = {
  light: '轻度',
  medium: '中度',
  deep: '深度',
}

const statusLabels: Record<DreamStatus, string> = {
  completed: '已完成',
  running: '运行中',
  failed: '失败',
}

interface DreamRecord {
  id: number
  time: string
  memories: number
  phases: DreamPhase[]
  status: DreamStatus
  weakened: number
  consolidated: number
}

export default function Dream() {
  const [intensity, setIntensity] = useState([50])
  const [maxMemories, setMaxMemories] = useState([500])
  const [autoDream, setAutoDream] = useState(false)
  const [scheduleInterval, setScheduleInterval] = useState('30')
  const [isRunning, setIsRunning] = useState(false)
  const [history, setHistory] = useState<DreamRecord[]>([
    {
      id: 1,
      time: '2分钟前',
      memories: 1247,
      phases: ['light', 'medium', 'deep'],
      status: 'completed',
      weakened: 12,
      consolidated: 1089,
    },
    {
      id: 2,
      time: '32分钟前',
      memories: 892,
      phases: ['light', 'deep'],
      status: 'completed',
      weakened: 8,
      consolidated: 756,
    },
    {
      id: 3,
      time: '1小时前',
      memories: 2103,
      phases: ['light', 'medium', 'deep', 'deep'],
      status: 'completed',
      weakened: 23,
      consolidated: 1876,
    },
  ])

  const totalCycles = 1247 + history.length
  const totalProcessed = history.reduce((s, h) => s + h.memories, 0)
  const totalWeakened = history.reduce((s, h) => s + h.weakened, 0)
  const totalConsolidated = history.reduce((s, h) => s + h.consolidated, 0)

  const autoDreamRef = useRef(false)
  const scheduleIntervalRef = useRef('30')

  useEffect(() => { autoDreamRef.current = autoDream }, [autoDream])
  useEffect(() => { scheduleIntervalRef.current = scheduleInterval }, [scheduleInterval])

  const handleRunDream = useCallback(async () => {
    setIsRunning(true)
    try {
      const res = await api.runDream()
      const d = res.data
      const phases: DreamPhase[] = ['light']
      if (d.paths_replayed > 0) phases.push('medium')
      if (d.memories_consolidated > 0) phases.push('deep')
      const newDream: DreamRecord = {
        id: Date.now(),
        time: new Date().toLocaleTimeString(),
        memories: d.paths_replayed,
        phases,
        status: 'completed',
        weakened: d.paths_weakened,
        consolidated: d.memories_consolidated,
      }
      setHistory((prev) => [newDream, ...prev])
    } catch {
      const newDream: DreamRecord = {
        id: Date.now(),
        time: new Date().toLocaleTimeString(),
        memories: 0,
        phases: ['light'],
        status: 'failed',
        weakened: 0,
        consolidated: 0,
      }
      setHistory((prev) => [newDream, ...prev])
    }
    setIsRunning(false)
  }, [])

  useEffect(() => {
    if (!autoDream) return
    const ms = Number(scheduleInterval) * 60 * 1000
    const timer = setInterval(() => {
      handleRunDream()
    }, ms)
    return () => clearInterval(timer)
  }, [autoDream, scheduleInterval, handleRunDream])

  return (
    <div className="relative min-h-[100dvh]">
      <div className="relative z-10 p-6">
        <motion.div
          variants={containerVariants}
          initial="hidden"
          animate="visible"
          className="mx-auto max-w-[1440px] space-y-6"
        >
          {/* ─── STAT CARDS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {[
              {
                label: '梦境周期数',
                value: totalCycles.toLocaleString(),
                icon: Moon,
                color: 'var(--accent-purple)',
              },
              {
                label: '处理记忆数',
                value: totalProcessed.toLocaleString(),
                icon: Activity,
                color: 'var(--dim-x)',
              },
              {
                label: '已弱化',
                value: totalWeakened.toLocaleString(),
                icon: TrendingDown,
                color: 'var(--accent-red)',
              },
              {
                label: '已巩固',
                value: totalConsolidated.toLocaleString(),
                icon: TrendingUp,
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

          {/* ─── RUN DREAM CYCLE ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-purple)26' }}
              >
                <Moon
                  className="h-5 w-5"
                  style={{ color: 'var(--accent-purple)' }}
                />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                运行梦境周期
              </h2>
            </div>

            <div className="grid grid-cols-1 gap-6 sm:grid-cols-3">
              <div>
                <label className="mb-2 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  强度 ({intensity[0]}%)
                </label>
                <Slider
                  value={intensity}
                  onValueChange={setIntensity}
                  min={10}
                  max={100}
                  step={5}
                />
              </div>
              <div>
                <label className="mb-2 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  最大记忆数 ({maxMemories[0]})
                </label>
                <Slider
                  value={maxMemories}
                  onValueChange={setMaxMemories}
                  min={100}
                  max={2000}
                  step={100}
                />
              </div>
              <div className="flex items-end">
                <Button
                  className="w-full"
                  disabled={isRunning}
                  style={{ backgroundColor: 'var(--accent-purple)' }}
                  onClick={handleRunDream}
                >
                  <Moon className="mr-2 h-4 w-4" />
                  {isRunning ? '运行中...' : '运行梦境周期'}
                </Button>
              </div>
            </div>

            {/* Auto-Dream */}
            <div className="mt-4 flex flex-wrap items-center gap-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
              <div className="flex items-center gap-2">
                <Switch checked={autoDream} onCheckedChange={setAutoDream} />
                <span className="font-body text-[13px] font-semibold text-[var(--text-primary)]">
                  自动梦境
                </span>
              </div>
              <div className="flex items-center gap-2">
                <Clock className="h-4 w-4 text-[var(--text-muted)]" />
                <span className="font-body text-[12px] text-[var(--text-secondary)]">
                  调度
                </span>
                <Select value={scheduleInterval} onValueChange={setScheduleInterval}>
                  <SelectTrigger className="h-7 w-[100px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="15">15 分钟</SelectItem>
                    <SelectItem value="30">30 分钟</SelectItem>
                    <SelectItem value="60">1 小时</SelectItem>
                    <SelectItem value="120">2 小时</SelectItem>
                  </SelectContent>
                </Select>
                <span className="font-body text-[12px] text-[var(--text-muted)]">
                  间隔
                </span>
              </div>
            </div>
          </motion.div>

          {/* ─── PHASE TIMELINE + ANALYTICS ─── */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            {/* Phase Timeline */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <h2 className="mb-4 font-display text-lg font-semibold text-[var(--text-primary)]">
                阶段时间线
              </h2>
              <div className="flex items-center gap-2">
                {(['light', 'medium', 'deep'] as DreamPhase[]).map((phase, i) => (
                  <div key={phase} className="flex flex-1 items-center gap-2">
                    <motion.div
                      initial={{ scaleY: 0 }}
                      animate={{ scaleY: 1 }}
                      transition={{ delay: i * 0.2, duration: 0.5, ease }}
                      className="flex flex-1 flex-col items-center gap-1"
                    >
                      <div
                        className="w-full rounded-t-lg"
                        style={{
                          height: `${40 + i * 30}px`,
                          backgroundColor:
                            phase === 'light'
                              ? 'var(--dim-x)'
                              : phase === 'medium'
                                ? 'var(--dim-z)'
                                : 'var(--accent-purple)',
                          opacity: 0.6,
                        }}
                      />
                      <span className="font-body text-[10px] font-semibold text-[var(--text-secondary)]">
                        {phaseLabels[phase]}
                      </span>
                    </motion.div>
                    {i < 2 && (
                      <div className="flex h-[2px] w-4 items-center">
                        <div className="h-full w-full bg-[var(--border-subtle)]" />
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </motion.div>

            {/* Phase Analytics */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <h2 className="mb-4 font-display text-lg font-semibold text-[var(--text-primary)]">
                阶段分析
              </h2>
              <div className="grid grid-cols-2 gap-4">
                <div className="rounded-lg bg-[var(--bg-surface)] p-4 text-center">
                  <TrendingDown className="mx-auto mb-2 h-6 w-6 text-[var(--accent-red)]" />
                  <p className="font-body text-[10px] text-[var(--text-muted)]">
                    弱化
                  </p>
                  <p className="font-mono text-xl font-bold text-[var(--accent-red)]">
                    {totalWeakened}
                  </p>
                </div>
                <div className="rounded-lg bg-[var(--bg-surface)] p-4 text-center">
                  <TrendingUp className="mx-auto mb-2 h-6 w-6 text-[var(--accent-green)]" />
                  <p className="font-body text-[10px] text-[var(--text-muted)]">
                    巩固
                  </p>
                  <p className="font-mono text-xl font-bold text-[var(--accent-green)]">
                    {totalConsolidated}
                  </p>
                </div>
              </div>
            </motion.div>
          </div>

          {/* ─── DREAM LOG ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                梦境日志
              </h2>
            </div>

            <div className="max-h-[360px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>ID</TableHead>
                    <TableHead>时间</TableHead>
                    <TableHead>记忆数</TableHead>
                    <TableHead>阶段</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {history.map((record) => (
                    <TableRow key={record.id}>
                      <TableCell className="font-mono text-xs">
                        #{record.id}
                      </TableCell>
                      <TableCell className="font-body text-[11px] text-[var(--text-muted)]">
                        {record.time}
                      </TableCell>
                      <TableCell className="font-mono text-xs">
                        {record.memories.toLocaleString()}
                      </TableCell>
                      <TableCell>
                        <div className="flex gap-1">
                          {record.phases.map((p, i) => (
                            <Badge
                              key={i}
                              variant="outline"
                              className="text-[9px]"
                              style={{
                                borderColor:
                                  p === 'light'
                                    ? 'var(--dim-x)'
                                    : p === 'medium'
                                      ? 'var(--dim-z)'
                                      : 'var(--accent-purple)',
                                color:
                                  p === 'light'
                                    ? 'var(--dim-x)'
                                    : p === 'medium'
                                      ? 'var(--dim-z)'
                                      : 'var(--accent-purple)',
                              }}
                            >
                              {phaseLabels[p]}
                            </Badge>
                          ))}
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant={
                            record.status === 'completed'
                              ? 'default'
                              : record.status === 'running'
                                ? 'secondary'
                                : 'destructive'
                          }
                          className="text-[10px]"
                        >
                          {statusLabels[record.status]}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex items-center justify-end gap-1">
                          <Button variant="ghost" size="sm" className="h-6 px-2 text-xs">
                            <RotateCcw className="mr-1 h-3 w-3" />
                            重放
                          </Button>
                          <Button variant="ghost" size="sm" className="h-6 px-2 text-xs text-[var(--accent-red)]">
                            <TrendingDown className="mr-1 h-3 w-3" />
                            弱化
                          </Button>
                          <Button variant="ghost" size="sm" className="h-6 px-2 text-xs text-[var(--accent-green)]">
                            <TrendingUp className="mr-1 h-3 w-3" />
                            巩固
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
