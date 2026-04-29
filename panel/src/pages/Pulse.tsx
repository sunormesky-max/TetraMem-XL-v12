import { useState, useCallback } from 'react'
import { motion } from 'framer-motion'
import {
  Zap,
  RotateCcw,
  Trash2,
  Target,
  TrendingUp,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
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

type PulseType = 'standard' | 'associative' | 'cascade'

interface PulseRecord {
  id: number
  type: PulseType
  origin: string
  reach: number
  energy: number
  time: string
}

const pulseTypeLabels: Record<PulseType, string> = {
  standard: '标准',
  associative: '关联',
  cascade: '级联',
}

const pulseTypeApiMap: Record<PulseType, string> = {
  standard: 'exploratory',
  associative: 'reinforcing',
  cascade: 'cascade',
}

function parseOrigin(input: string): [number, number, number] {
  if (!input.trim()) return [0, 0, 0]
  const parts = input.split(',').map((s) => parseInt(s.trim(), 10))
  if (parts.length === 3 && parts.every((p) => !isNaN(p))) {
    return parts as [number, number, number]
  }
  return [0, 0, 0]
}

export default function Pulse() {
  const [pulseType, setPulseType] = useState<PulseType>('standard')
  const [originNode, setOriginNode] = useState('')
  const [history, setHistory] = useState<PulseRecord[]>([
    {
      id: 1,
      type: 'associative',
      origin: '4521847',
      reach: 847,
      energy: 3.24,
      time: '2秒前',
    },
    {
      id: 2,
      type: 'standard',
      origin: '4521001',
      reach: 124,
      energy: 1.56,
      time: '15秒前',
    },
    {
      id: 3,
      type: 'cascade',
      origin: '4520000',
      reach: 3214,
      energy: 8.91,
      time: '1分钟前',
    },
    {
      id: 4,
      type: 'associative',
      origin: '4521500',
      reach: 512,
      energy: 2.78,
      time: '3分钟前',
    },
  ])

  const totalPulses = history.length + 1247
  const avgReach = Math.round(
    (history.reduce((s, h) => s + h.reach, 0) / history.length) * 10
  ) / 10
  const avgEnergy =
    Math.round(
      (history.reduce((s, h) => s + h.energy, 0) / history.length) * 100
    ) / 100

  const handleFirePulse = useCallback(async () => {
    const source = parseOrigin(originNode)
    const apiPulseType = pulseTypeApiMap[pulseType]
    try {
      const res = await api.firePulse(source, apiPulseType)
      const d = res.data
      const newPulse: PulseRecord = {
        id: history.length + 1,
        type: pulseType,
        origin: originNode || source.join(','),
        reach: d.visited_nodes,
        energy: d.final_strength,
        time: '刚刚',
      }
      setHistory((prev) => [newPulse, ...prev])
    } catch {
      const newPulse: PulseRecord = {
        id: history.length + 1,
        type: pulseType,
        origin: originNode || source.join(','),
        reach: 0,
        energy: 0,
        time: '刚刚 (失败)',
      }
      setHistory((prev) => [newPulse, ...prev])
    }
  }, [pulseType, originNode, history.length])

  const handleClear = useCallback(() => {
    setHistory([])
  }, [])

  const handleReplay = useCallback((record: PulseRecord) => {
    const replayed: PulseRecord = {
      ...record,
      id: history.length + 1,
      time: '刚刚 (重播)',
    }
    setHistory((prev) => [replayed, ...prev])
  }, [history.length])

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
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {[
              {
                label: '脉冲触发数',
                value: totalPulses.toLocaleString(),
                icon: Zap,
                color: 'var(--dim-e)',
              },
              {
                label: '平均覆盖',
                value: `${avgReach.toFixed(0)} 节点`,
                icon: TrendingUp,
                color: 'var(--dim-x)',
              },
              {
                label: '平均能量',
                value: avgEnergy.toFixed(2),
                icon: Target,
                color: 'var(--accent-purple)',
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

          {/* ─── FIRE PULSE PANEL ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--dim-e)26' }}
              >
                <Zap className="h-5 w-5" style={{ color: 'var(--dim-e)' }} />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                触发脉冲
              </h2>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  脉冲类型
                </label>
                <Select
                  value={pulseType}
                  onValueChange={(v) => setPulseType(v as PulseType)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="standard">标准</SelectItem>
                    <SelectItem value="associative">关联</SelectItem>
                    <SelectItem value="cascade">级联</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  起源节点
                </label>
                <div className="flex gap-2">
                  <Input
                    placeholder="x,y,z 坐标"
                    value={originNode}
                    onChange={(e) => setOriginNode(e.target.value)}
                    className="font-mono text-xs"
                  />
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() =>
                      setOriginNode(
                        `${Math.floor(Math.random() * 20)},${Math.floor(Math.random() * 20)},${Math.floor(Math.random() * 20)}`
                      )
                    }
                  >
                    随机
                  </Button>
                </div>
              </div>
              <div className="flex items-end">
                <Button
                  className="w-full"
                  style={{ backgroundColor: 'var(--dim-e)' }}
                  onClick={handleFirePulse}
                >
                  <Zap className="mr-2 h-4 w-4" />
                  触发脉冲
                </Button>
              </div>
            </div>
          </motion.div>

          {/* ─── DISTRIBUTION + HISTORY ─── */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_2fr]">
            {/* Distribution */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <h2 className="mb-4 font-display text-lg font-semibold text-[var(--text-primary)]">
                分布
              </h2>
              <div className="space-y-4">
                {[
                  { type: 'standard' as PulseType, count: 523, pct: 42 },
                  { type: 'associative' as PulseType, count: 612, pct: 49 },
                  { type: 'cascade' as PulseType, count: 112, pct: 9 },
                ].map((item) => (
                  <div key={item.type}>
                    <div className="mb-1 flex items-center justify-between">
                      <span className="font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                        {pulseTypeLabels[item.type]}
                      </span>
                      <span className="font-mono text-[11px] text-[var(--text-muted)]">
                        {item.count} ({item.pct}%)
                      </span>
                    </div>
                    <div className="h-2 w-full overflow-hidden rounded-full bg-[var(--bg-surface)]">
                      <motion.div
                        initial={{ width: 0 }}
                        animate={{ width: `${item.pct}%` }}
                        transition={{ duration: 0.8, ease }}
                        className="h-full rounded-full"
                        style={{
                          background:
                            item.type === 'standard'
                              ? 'var(--dim-e)'
                              : item.type === 'associative'
                                ? 'var(--dim-x)'
                                : 'var(--accent-purple)',
                        }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </motion.div>

            {/* Pulse History */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="font-display text-lg font-semibold text-[var(--text-primary)]">
                  脉冲历史
                </h2>
                <div className="flex gap-2">
                  <Button variant="ghost" size="sm" onClick={handleClear}>
                    <Trash2 className="mr-1.5 h-3.5 w-3.5" />
                    清空
                  </Button>
                </div>
              </div>

              <div className="max-h-[360px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>ID</TableHead>
                      <TableHead>类型</TableHead>
                      <TableHead>起源</TableHead>
                      <TableHead>覆盖</TableHead>
                      <TableHead>能量</TableHead>
                      <TableHead>时间</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {history.map((record) => (
                      <TableRow key={record.id}>
                        <TableCell className="font-mono text-xs">
                          #{record.id}
                        </TableCell>
                        <TableCell>
                          <Badge
                            variant="outline"
                            className="text-[10px]"
                            style={{
                              borderColor:
                                record.type === 'standard'
                                  ? 'var(--dim-e)'
                                  : record.type === 'associative'
                                    ? 'var(--dim-x)'
                                    : 'var(--accent-purple)',
                              color:
                                record.type === 'standard'
                                  ? 'var(--dim-e)'
                                  : record.type === 'associative'
                                    ? 'var(--dim-x)'
                                    : 'var(--accent-purple)',
                            }}
                          >
                            {pulseTypeLabels[record.type]}
                          </Badge>
                        </TableCell>
                        <TableCell className="font-mono text-xs">
                          {record.origin}
                        </TableCell>
                        <TableCell className="font-mono text-xs">
                          {record.reach}
                        </TableCell>
                        <TableCell className="font-mono text-xs text-[var(--dim-e)]">
                          {record.energy.toFixed(2)}
                        </TableCell>
                        <TableCell className="font-body text-[11px] text-[var(--text-muted)]">
                          {record.time}
                        </TableCell>
                        <TableCell className="text-right">
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 px-2 text-xs"
                            onClick={() => handleReplay(record)}
                          >
                            <RotateCcw className="mr-1 h-3 w-3" />
                            重播
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </motion.div>
          </div>
        </motion.div>
      </div>
    </div>
  )
}
