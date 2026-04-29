import { useState, useCallback, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Activity,
  ArrowUp,
  ArrowDown,
  Maximize,
  Scale,
  AlertTriangle,
  Save,
  Shield,
  CheckCircle,
  Clock,
  RotateCcw,
  XCircle,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { api, type StatsData } from '../services/api'
import type { BackupInfo } from '../services/api'

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

interface Strategy {
  key: string
  label: string
  icon: typeof ArrowUp
  desc: string
  active: boolean
}

interface RegulationRecord {
  id: number
  type: string
  result: string
  time: string
}

export default function Regulation() {
  const [strategies, setStrategies] = useState<Strategy[]>([
    { key: 'scale-up', label: '放大', icon: ArrowUp, desc: '增加晶格维度', active: true },
    { key: 'scale-down', label: '缩小', icon: ArrowDown, desc: '减少晶格维度', active: false },
    { key: 'scale-to-fit', label: '自适应', icon: Maximize, desc: '动态匹配负载', active: true },
    { key: 'load-balance', label: '负载均衡', icon: Scale, desc: '均匀分布压力', active: true },
    { key: 'emergency-scale', label: '紧急缩放', icon: AlertTriangle, desc: '应急扩容', active: false },
  ])

  const [backups, setBackups] = useState<BackupInfo[]>([])
  const [backupLoading, setBackupLoading] = useState(false)
  const [backupError, setBackupError] = useState('')

  const [history, setHistory] = useState<RegulationRecord[]>([
    { id: 1, type: '压力归一化', result: '成功', time: '5分钟前' },
    { id: 2, type: '负载均衡', result: '成功', time: '15分钟前' },
    { id: 3, type: '自动缩放', result: '成功', time: '30分钟前' },
    { id: 4, type: '世代轮换', result: '成功', time: '1小时前' },
  ])

  const [verified, setVerified] = useState(false)
  const [verifying, setVerifying] = useState(false)
  const [stats, setStats] = useState<StatsData['data'] | null>(null)

  useEffect(() => {
    api.getStats().then((res) => {
      if (res.success) setStats(res.data)
    }).catch(() => {})
    api.listBackups().then((res) => {
      if (res.success && res.data) setBackups(res.data)
    }).catch(() => {})
  }, [])

  const handleCreateBackup = useCallback(async () => {
    setBackupLoading(true)
    setBackupError('')
    try {
      const res = await api.createBackup()
      if (res.success) {
        const listRes = await api.listBackups()
        if (listRes.success && listRes.data) setBackups(listRes.data)
      }
    } catch (err: any) {
      setBackupError(err.message || '备份失败')
    }
    setBackupLoading(false)
  }, [])

  const toggleStrategy = useCallback((key: string) => {
    setStrategies((prev) =>
      prev.map((s) => (s.key === key ? { ...s, active: !s.active } : s))
    )
  }, [])

  const handleVerify = useCallback(async () => {
    setVerifying(true)
    try {
      const res = await api.getStats()
      if (res.success) {
        setStats(res.data)
        setVerified(res.data.conservation_ok)
        setTimeout(() => setVerified(false), 3000)
      }
    } catch {
      setVerified(false)
    }
    setVerifying(false)
  }, [])

  const handleAutoScale = useCallback(async () => {
    try {
      const res = await api.autoScale()
      if (res.success) {
        const d = res.data
        const record: RegulationRecord = {
          id: history.length + 1,
          type: `自动缩放: ${d.reason}`,
          result: `+${d.nodes_added} -${d.nodes_removed}`,
          time: '刚刚',
        }
        setHistory((prev) => [record, ...prev])
      }
    } catch {}
  }, [history.length])

  const handleRegulate = useCallback(async () => {
    try {
      const res = await api.regulate()
      if (res.success) {
        for (const action of res.data) {
          setHistory((prev) => [
            {
              id: prev.length + 1,
              type: '调节',
              result: action,
              time: '刚刚',
            },
            ...prev,
          ])
        }
      }
    } catch {}
  }, [])

  const activeCount = strategies.filter((s) => s.active).length

  const dimPressures = [
    { dim: 'X', value: stats ? Math.round((stats.even / stats.nodes) * 100) : 12 },
    { dim: 'Y', value: stats ? Math.round((stats.odd / stats.nodes) * 100) : 8 },
    { dim: 'Z', value: stats ? Math.round((stats.dark / stats.nodes) * 100) : 15 },
    { dim: 'E', value: stats ? Math.round(stats.utilization * 100) : 6 },
    { dim: 'S', value: stats ? Math.round(((stats.allocated_energy) / stats.total_energy) * 100) : 22 },
    { dim: 'T', value: stats ? Math.round((stats.physical_energy / stats.total_energy) * 100) : 10 },
    { dim: '\u03bc', value: stats ? Math.round((stats.dark_energy / stats.total_energy) * 100) : 4 },
  ]
  const avgPressure = Math.round(
    dimPressures.reduce((s, d) => s + d.value, 0) / dimPressures.length
  )

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
                label: '平均压力',
                value: `${avgPressure}%`,
                icon: Activity,
                color: 'var(--dim-t)',
              },
              {
                label: '自动缩放',
                value: `${activeCount}/5`,
                icon: Maximize,
                color: 'var(--dim-x)',
              },
              {
                label: '备份数',
                value: String(backups.length),
                icon: Save,
                color: 'var(--dim-z)',
              },
              {
                label: '守恒',
                value: verified ? '已验证' : stats ? (stats.conservation_ok ? '正常' : '异常') : '--',
                icon: Shield,
                color: verified || (stats?.conservation_ok ?? false) ? 'var(--accent-green)' : 'var(--accent-red)',
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

          {/* ─── DIMENSION PRESSURE ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
              维度压力
            </h2>
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-4 lg:grid-cols-7">
              {dimPressures.map((dp) => (
                <div
                  key={dp.dim}
                  className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-3 text-center"
                >
                  <p className="font-mono text-sm font-bold text-[var(--dim-x)]">
                    {dp.dim}
                  </p>
                  <div className="mx-auto my-2 h-16 w-2 overflow-hidden rounded-full bg-[var(--bg-deep)]">
                    <motion.div
                      initial={{ height: 0 }}
                      animate={{ height: `${dp.value}%` }}
                      transition={{ duration: 0.8, ease }}
                      className="w-full rounded-full"
                      style={{
                        backgroundColor:
                          dp.value > 20
                            ? 'var(--accent-red)'
                            : dp.value > 12
                              ? 'var(--accent-amber)'
                              : 'var(--accent-green)',
                      }}
                    />
                  </div>
                  <p className="font-mono text-[10px] text-[var(--text-secondary)]">
                    {dp.value}%
                  </p>
                </div>
              ))}
            </div>
          </motion.div>

          {/* ─── AUTO-SCALE STRATEGIES ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
              自动缩放策略
            </h2>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {strategies.map((s) => {
                const Icon = s.icon
                return (
                  <motion.div
                    key={s.key}
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                    onClick={() => toggleStrategy(s.key)}
                    className="flex cursor-pointer items-center gap-3 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4 transition-colors"
                    style={{
                      borderColor: s.active
                        ? 'var(--accent-green)'
                        : 'var(--border-subtle)',
                    }}
                  >
                    <div
                      className="flex h-8 w-8 items-center justify-center rounded-full"
                      style={{
                        backgroundColor: s.active
                          ? 'var(--accent-green)26'
                          : 'var(--bg-elevated)',
                      }}
                    >
                      <Icon
                        className="h-4 w-4"
                        style={{
                          color: s.active
                            ? 'var(--accent-green)'
                            : 'var(--text-muted)',
                        }}
                      />
                    </div>
                    <div className="flex-1">
                      <p
                        className="font-body text-[13px] font-semibold"
                        style={{
                          color: s.active
                            ? 'var(--text-primary)'
                            : 'var(--text-muted)',
                        }}
                      >
                        {s.label}
                      </p>
                      <p className="font-body text-[10px] text-[var(--text-muted)]">
                        {s.desc}
                      </p>
                    </div>
                    {s.active && (
                      <CheckCircle className="h-4 w-4 text-[var(--accent-green)]" />
                    )}
                  </motion.div>
                )
              })}
            </div>
            <div className="mt-4 flex gap-2">
              <Button
                style={{ backgroundColor: 'var(--dim-x)' }}
                onClick={handleAutoScale}
              >
                <Maximize className="mr-2 h-4 w-4" />
                执行自动缩放
              </Button>
              <Button
                style={{ backgroundColor: 'var(--dim-t)' }}
                onClick={handleRegulate}
              >
                <Activity className="mr-2 h-4 w-4" />
                调节
              </Button>
            </div>
          </motion.div>

          {/* ─── ENERGY CONSERVATION + BACKUPS ─── */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            {/* Energy Conservation */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
                能量守恒验证
              </h2>
              <div className="flex flex-col items-center justify-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-deep)] p-8">
                {verified ? (
                  <motion.div
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    className="text-center"
                  >
                    <CheckCircle className="mx-auto mb-3 h-16 w-16 text-[var(--accent-green)]" />
                    <p className="font-display text-lg font-semibold text-[var(--accent-green)]">
                      能量守恒已验证
                    </p>
                    <p className="mt-1 font-mono text-sm text-[var(--text-secondary)]">
                      {stats ? `利用率 ${(stats.utilization * 100).toFixed(1)}%` : '零损耗 · 100% 守恒'}
                    </p>
                  </motion.div>
                ) : (
                  <>
                    <Shield className="mb-3 h-12 w-12 text-[var(--dim-e)] opacity-50" />
                    <p className="mb-4 font-body text-sm text-[var(--text-muted)]">
                      验证系统能量守恒
                    </p>
                    <Button
                      disabled={verifying}
                      style={{ backgroundColor: 'var(--accent-green)' }}
                      onClick={handleVerify}
                    >
                      <Shield className="mr-2 h-4 w-4" />
                      {verifying ? '验证中...' : '验证守恒'}
                    </Button>
                  </>
                )}
              </div>
            </motion.div>

            {/* Backup Management */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  备份管理
                </h2>
                <Button size="sm" variant="outline" disabled={backupLoading} onClick={handleCreateBackup}>
                  <Save className="mr-1.5 h-3.5 w-3.5" />
                  {backupLoading ? '备份中...' : '创建备份'}
                </Button>
              </div>

              {backupError && (
                <div className="mb-3 flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                  <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
                  <span className="font-body text-[12px] text-[var(--accent-red)]">
                    {backupError}
                  </span>
                </div>
              )}

              {backups.length === 0 ? (
                <div className="py-6 text-center">
                  <Save className="mx-auto mb-2 h-8 w-8 text-[var(--text-muted)] opacity-40" />
                  <p className="font-body text-[12px] text-[var(--text-muted)]">
                    暂无备份，点击"创建备份"开始
                  </p>
                </div>
              ) : (
                <div className="max-h-[200px] overflow-auto space-y-2">
                  {backups.map((b) => (
                    <div
                      key={b.id}
                      className="flex items-center gap-3 rounded-lg bg-[var(--bg-surface)] px-4 py-3"
                    >
                      <Save className="h-4 w-4 text-[var(--dim-z)]" />
                      <div className="flex-1">
                        <p className="font-mono text-[11px] text-[var(--text-primary)]">
                          Backup #{b.id} · Gen {b.generation}
                        </p>
                        <p className="font-body text-[10px] text-[var(--text-muted)]">
                          {b.node_count} nodes · {b.memory_count} mems · {(b.bytes / 1024).toFixed(1)} KB
                        </p>
                      </div>
                      <Badge variant={b.conservation_ok ? 'default' : 'destructive'} className="text-[8px]">
                        {b.trigger}
                      </Badge>
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        {new Date(b.timestamp_ms).toLocaleTimeString()}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </motion.div>
          </div>

          {/* ─── REGULATION HISTORY ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
              调节历史
            </h2>
            <div className="max-h-[280px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>ID</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>结果</TableHead>
                    <TableHead>时间</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {history.map((r) => (
                    <TableRow key={r.id}>
                      <TableCell className="font-mono text-xs">
                        #{r.id}
                      </TableCell>
                      <TableCell className="font-body text-[12px]">
                        {r.type}
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant="outline"
                          className="border-[var(--accent-green)] text-[10px] text-[var(--accent-green)]"
                        >
                          {r.result}
                        </Badge>
                      </TableCell>
                      <TableCell className="flex items-center gap-1 font-body text-[11px] text-[var(--text-muted)]">
                        <Clock className="h-3 w-3" />
                        {r.time}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </motion.div>

          {/* ─── GENERATIONAL ROTATION ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <h2 className="mb-4 font-display text-lg font-semibold text-[var(--text-primary)]">
              世代轮换
            </h2>
            <div className="flex items-center gap-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
              <div className="flex-1">
                <div className="mb-2 flex justify-between">
                  <span className="font-body text-[12px] text-[var(--text-secondary)]">
                    当前世代
                  </span>
                  <span className="font-mono text-[12px] text-[var(--dim-x)]">
                    Gen #847
                  </span>
                </div>
                <div className="h-2 w-full overflow-hidden rounded-full bg-[var(--bg-deep)]">
                  <motion.div
                    initial={{ width: 0 }}
                    animate={{ width: '68%' }}
                    transition={{ duration: 1.2, ease }}
                    className="h-full rounded-full"
                    style={{
                      background:
                        'linear-gradient(90deg, var(--dim-x), var(--accent-green))',
                    }}
                  />
                </div>
              </div>
              <Button variant="outline" size="sm">
                <RotateCcw className="mr-1.5 h-3.5 w-3.5" />
                轮换
              </Button>
            </div>
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
