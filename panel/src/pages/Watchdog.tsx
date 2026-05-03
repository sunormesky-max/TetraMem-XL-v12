import { useState, useCallback, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Shield,
  Heart,
  Eye,
  AlertTriangle,
  Activity,
  CheckCircle,
  XCircle,
  Clock,
  Server,
  BookOpen,
  Radio,
  Loader2,
  Zap,
  Gauge,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  api,
  type WatchdogStatusResult,
  type WatchdogCheckupResult,
  type ClusteringStatusResult,
  type ConstitutionStatusResult,
  type EventsStatusResult,
  type StatsData,
} from '../services/api'

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

function formatUptime(ms: number): string {
  const hours = Math.floor(ms / 3600000)
  const minutes = Math.floor((ms % 3600000) / 60000)
  if (hours > 0) return `${hours} 小时 ${minutes} 分钟`
  return `${minutes} 分钟`
}

const levelConfig: Record<string, { label: string; color: string; bg: string }> = {
  healthy: { label: '健康', color: 'var(--accent-green)', bg: 'var(--accent-green)26' },
  warning: { label: '警告', color: 'var(--accent-amber, #f59e0b)', bg: 'rgba(245,158,11,0.15)' },
  critical: { label: '严重', color: 'var(--accent-red)', bg: 'var(--accent-red)26' },
}

export default function Watchdog() {
  const [status, setStatus] = useState<WatchdogStatusResult['data'] | null>(null)
  const [checkup, setCheckup] = useState<WatchdogCheckupResult['data'] | null>(null)
  const [checkupLoading, setCheckupLoading] = useState(false)

  const [clustering, setClustering] = useState<ClusteringStatusResult['data'] | null>(null)
  const [constitution, setConstitution] = useState<ConstitutionStatusResult['data'] | null>(null)
  const [events, setEvents] = useState<EventsStatusResult['data'] | null>(null)

  const [conservation, setConservation] = useState<{ conservation_ok: boolean; energy_drift: number } | null>(null)
  const [conservationLoading, setConservationLoading] = useState(false)

  const [systemError, setSystemError] = useState('')

  useEffect(() => {
    api.watchdogStatus().then((res) => {
      if (res.success) setStatus(res.data)
    }).catch(() => {})

    Promise.all([
      api.clusteringStatus().catch(() => null),
      api.constitutionStatus().catch(() => null),
      api.eventsStatus().catch(() => null),
    ]).then(([clusterRes, constRes, eventRes]) => {
      if (clusterRes?.success) setClustering(clusterRes.data)
      if (constRes?.success) setConstitution(constRes.data)
      if (eventRes?.success) setEvents(eventRes.data)
      if (!clusterRes && !constRes && !eventRes) {
        setSystemError('无法加载系统状态')
      }
    })
  }, [])

  const handleCheckup = useCallback(async () => {
    setCheckupLoading(true)
    try {
      const res = await api.watchdogCheckup()
      if (res.success) {
        setCheckup(res.data)
      }
    } catch {}
    setCheckupLoading(false)
  }, [])

  const handleConservation = useCallback(async () => {
    setConservationLoading(true)
    try {
      const res = await api.getStats()
      if (res.success) {
        setConservation({
          conservation_ok: res.data.conservation_ok,
          energy_drift: res.data.total_energy > 0
            ? Math.abs(res.data.allocated_energy + res.data.available_energy - res.data.total_energy) / res.data.total_energy
            : 0,
        })
      }
    } catch {}
    setConservationLoading(false)
  }, [])

  const lvl = checkup ? levelConfig[checkup.level] ?? levelConfig.healthy : null

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
          <motion.div variants={cardVariants}>
            <div className="flex items-center gap-3 mb-2">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-cyan)26' }}
              >
                <Shield className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <h1 className="font-display text-2xl font-bold text-[var(--text-primary)]">
                看门狗
              </h1>
            </div>
            <p className="font-body text-[13px] text-[var(--text-muted)]">
              系统健康监控与诊断面板 — 实时追踪能量守恒、模块状态与系统稳定性
            </p>
          </motion.div>

          {/* ─── HEALTH OVERVIEW CARDS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            {[
              {
                label: '总检查次数',
                value: status?.total_checkups?.toLocaleString() ?? '--',
                icon: Eye,
                color: 'var(--accent-cyan)',
              },
              {
                label: '运行时间',
                value: status ? formatUptime(status.uptime_ms) : '--',
                icon: Clock,
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

          {/* ─── RUN CHECKUP ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--accent-cyan)26' }}
                >
                  <Activity className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
                </div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  系统体检
                </h2>
              </div>
              <Button
                disabled={checkupLoading}
                style={{ backgroundColor: 'var(--accent-cyan)' }}
                onClick={handleCheckup}
              >
                {checkupLoading ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Heart className="mr-2 h-4 w-4" />
                )}
                {checkupLoading ? '检查中...' : '执行体检'}
              </Button>
            </div>

            {checkup && (
              <motion.div
                initial={{ opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4, ease }}
                className="space-y-4"
              >
                {/* Level & Utilization */}
                <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                    <div className="mb-2 flex items-center justify-between">
                      <span className="font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                        健康等级
                      </span>
                      <Badge
                        variant="outline"
                        className="text-[10px]"
                        style={{
                          borderColor: lvl?.color,
                          color: lvl?.color,
                          backgroundColor: lvl?.bg,
                        }}
                      >
                        {lvl?.label ?? checkup.level}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-2">
                      {checkup.level === 'healthy' ? (
                        <CheckCircle className="h-5 w-5" style={{ color: 'var(--accent-green)' }} />
                      ) : checkup.level === 'critical' ? (
                        <XCircle className="h-5 w-5" style={{ color: 'var(--accent-red)' }} />
                      ) : (
                        <AlertTriangle className="h-5 w-5" style={{ color: 'var(--accent-amber, #f59e0b)' }} />
                      )}
                      <span className="font-mono text-lg font-bold text-[var(--text-primary)]">
                        {lvl?.label ?? checkup.level}
                      </span>
                    </div>
                  </div>

                  <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                    <div className="mb-2 flex items-center justify-between">
                      <span className="font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                        资源利用率
                      </span>
                      <span className="font-mono text-[12px] text-[var(--text-muted)]">
                        {(checkup.utilization * 100).toFixed(1)}%
                      </span>
                    </div>
                    <div className="h-3 w-full overflow-hidden rounded-full bg-[var(--bg-deep)]">
                      <motion.div
                        initial={{ width: 0 }}
                        animate={{ width: `${checkup.utilization * 100}%` }}
                        transition={{ duration: 0.8, ease }}
                        className="h-full rounded-full"
                        style={{
                          backgroundColor:
                            checkup.utilization > 0.9
                              ? 'var(--accent-red)'
                              : checkup.utilization > 0.7
                                ? 'var(--accent-amber, #f59e0b)'
                                : 'var(--accent-green)',
                        }}
                      />
                    </div>
                  </div>
                </div>

                {/* Conservation OK */}
                <div className="flex items-center gap-3 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  {checkup.conservation_ok ? (
                    <CheckCircle className="h-5 w-5" style={{ color: 'var(--accent-green)' }} />
                  ) : (
                    <XCircle className="h-5 w-5" style={{ color: 'var(--accent-red)' }} />
                  )}
                  <span className="font-body text-[13px] font-semibold text-[var(--text-primary)]">
                    守恒状态
                  </span>
                  <Badge
                    variant="outline"
                    className="text-[10px]"
                    style={{
                      borderColor: checkup.conservation_ok ? 'var(--accent-green)' : 'var(--accent-red)',
                      color: checkup.conservation_ok ? 'var(--accent-green)' : 'var(--accent-red)',
                    }}
                  >
                    {checkup.conservation_ok ? '正常' : '异常'}
                  </Badge>
                </div>

                {/* Actions */}
                {checkup.actions.length > 0 && (
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                    <p className="mb-2 font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                      建议操作
                    </p>
                    <div className="space-y-2">
                      {checkup.actions.map((action, i) => (
                        <div
                          key={i}
                          className="flex items-start gap-2 rounded-md bg-[var(--bg-deep)] px-3 py-2"
                        >
                          <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" style={{ color: 'var(--accent-amber, #f59e0b)' }} />
                          <span className="font-body text-[12px] text-[var(--text-primary)]">
                            {action}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </motion.div>
            )}

            {!checkup && !checkupLoading && (
              <div className="flex flex-col items-center justify-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-deep)] py-10">
                <Heart className="mb-3 h-10 w-10 text-[var(--text-muted)] opacity-30" />
                <p className="font-body text-[13px] text-[var(--text-muted)]">
                  点击"执行体检"检查系统健康状态
                </p>
              </div>
            )}
          </motion.div>

          {/* ─── SYSTEM STATUS GRID ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {[
              {
                title: '集群状态',
                icon: Server,
                color: 'var(--accent-cyan)',
                data: clustering,
                items: clustering
                  ? [
                      { label: '已聚类记忆', value: String(clustering.memories_clustered) },
                      { label: '吸引子', value: String(clustering.attractors_found) },
                      { label: '活跃隧道', value: String(clustering.tunnels_active) },
                      { label: '活跃桥接', value: String(clustering.bridges_active) },
                    ]
                  : [],
              },
              {
                title: '宪法规则',
                icon: BookOpen,
                color: 'var(--accent-green)',
                data: constitution,
                items: constitution
                  ? [
                      { label: '规则数', value: String(constitution.rules_count) },
                      { label: '边界数', value: String(constitution.bounds_count) },
                    ]
                  : [],
              },
              {
                title: '事件总线',
                icon: Radio,
                color: 'var(--accent-cyan)',
                data: events,
                items: events
                  ? [
                      { label: '历史长度', value: String(events.history_len) },
                      { label: '订阅者', value: String(events.subscriber_count) },
                    ]
                  : [],
              },
            ].map((card) => (
              <motion.div
                key={card.title}
                variants={cardVariants}
                className="glass-panel p-6"
              >
                <div className="mb-4 flex items-center gap-3">
                  <div
                    className="flex h-9 w-9 items-center justify-center rounded-full"
                    style={{ backgroundColor: `${card.color}26` }}
                  >
                    <card.icon className="h-4 w-4" style={{ color: card.color }} />
                  </div>
                  <h3 className="font-display text-lg font-semibold text-[var(--text-primary)]">
                    {card.title}
                  </h3>
                  <Badge
                    variant="outline"
                    className="ml-auto text-[9px]"
                    style={{
                      borderColor: card.data ? 'var(--accent-green)' : 'var(--text-muted)',
                      color: card.data ? 'var(--accent-green)' : 'var(--text-muted)',
                    }}
                  >
                    {card.data ? '在线' : '离线'}
                  </Badge>
                </div>

                {card.data && card.items.length > 0 ? (
                  <div className="space-y-3">
                    {card.items.map((item) => (
                      <div key={item.label} className="flex items-center justify-between">
                        <span className="font-body text-[12px] text-[var(--text-secondary)]">
                          {item.label}
                        </span>
                        <span className="font-mono text-[13px] font-semibold text-[var(--text-primary)]">
                          {item.value}
                        </span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="py-4 text-center">
                    <p className="font-body text-[11px] text-[var(--text-muted)]">
                      暂无数据
                    </p>
                  </div>
                )}
              </motion.div>
            ))}
          </div>

          {/* ─── CONSERVATION CHECK ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--accent-green)26' }}
                >
                  <Gauge className="h-5 w-5" style={{ color: 'var(--accent-green)' }} />
                </div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  守恒检查
                </h2>
              </div>
              <Button
                variant="outline"
                disabled={conservationLoading}
                onClick={handleConservation}
              >
                {conservationLoading ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Zap className="mr-2 h-4 w-4" />
                )}
                {conservationLoading ? '检查中...' : '检查守恒'}
              </Button>
            </div>

            {conservation ? (
              <motion.div
                initial={{ opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4, ease }}
                className="grid grid-cols-1 gap-4 sm:grid-cols-2"
              >
                <div className="flex items-center gap-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  {conservation.conservation_ok ? (
                    <CheckCircle className="h-8 w-8 shrink-0" style={{ color: 'var(--accent-green)' }} />
                  ) : (
                    <XCircle className="h-8 w-8 shrink-0" style={{ color: 'var(--accent-red)' }} />
                  )}
                  <div>
                    <p className="font-body text-[12px] text-[var(--text-secondary)]">
                      守恒状态
                    </p>
                    <p
                      className="font-mono text-lg font-bold"
                      style={{
                        color: conservation.conservation_ok ? 'var(--accent-green)' : 'var(--accent-red)',
                      }}
                    >
                      {conservation.conservation_ok ? '守恒正常' : '守恒异常'}
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  <Activity className="h-8 w-8 shrink-0" style={{ color: 'var(--accent-cyan)' }} />
                  <div>
                    <p className="font-body text-[12px] text-[var(--text-secondary)]">
                      能量漂移
                    </p>
                    <p className="font-mono text-lg font-bold text-[var(--text-primary)]">
                      {(conservation.energy_drift * 100).toFixed(4)}%
                    </p>
                  </div>
                </div>
              </motion.div>
            ) : (
              <div className="flex flex-col items-center justify-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-deep)] py-8">
                <Gauge className="mb-3 h-8 w-8 text-[var(--text-muted)] opacity-30" />
                <p className="font-body text-[13px] text-[var(--text-muted)]">
                  点击"检查守恒"验证能量守恒状态
                </p>
              </div>
            )}
          </motion.div>

          {systemError && (
            <motion.div
              variants={cardVariants}
              className="flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-4 py-3"
            >
              <AlertTriangle className="h-4 w-4 text-[var(--accent-red)]" />
              <span className="font-body text-[12px] text-[var(--accent-red)]">
                {systemError}
              </span>
            </motion.div>
          )}
        </motion.div>
      </div>
    </div>
  )
}
