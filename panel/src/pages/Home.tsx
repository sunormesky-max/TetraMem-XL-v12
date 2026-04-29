import { useState, useEffect, useCallback, useRef } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import {
  Hexagon,
  Database,
  Zap,
  Target,
  CheckCircle,
  Activity,
  Moon,
  Plus,
  Trash2,
  Save,
  Eye,
  ChevronRight,
  AlertTriangle,
} from 'lucide-react'
import {
  RadialBarChart,
  RadialBar,
  ResponsiveContainer,
  PolarAngleAxis,
} from 'recharts'
import ParticleBackground from '../components/ParticleBackground'
import { useNavigate } from 'react-router-dom'
import { api, type StatsData, type HealthData } from '../services/api'

const ease = [0.16, 1, 0.3, 1] as [number, number, number, number]

/* ─────────────────────── MODULE STATUS DATA ─────────────────────── */
type ModuleStatus = 'active' | 'standby' | 'processing'

interface ModuleItem {
  name: string
  status: ModuleStatus
  lines: string
}

const modules: ModuleItem[] = [
  // Core Layer
  { name: 'coord', status: 'active', lines: '1.2K' },
  { name: 'energy', status: 'active', lines: '2.8K' },
  { name: 'lattice', status: 'active', lines: '3.1K' },
  { name: 'node', status: 'processing', lines: '4.5K' },
  // Memory Layer
  { name: 'memory', status: 'active', lines: '5.2K' },
  { name: 'hebbian', status: 'active', lines: '1.8K' },
  { name: 'pulse', status: 'processing', lines: '2.4K' },
  { name: 'dream', status: 'active', lines: '3.6K' },
  // Cognitive Layer
  { name: 'crystal', status: 'active', lines: '2.1K' },
  { name: 'topology', status: 'active', lines: '1.5K' },
  { name: 'reasoning', status: 'standby', lines: '890' },
  // Adaptive Layer
  { name: 'autoscale', status: 'active', lines: '1.1K' },
  { name: 'regulation', status: 'active', lines: '2.3K' },
  // Safety Layer
  { name: 'observer', status: 'active', lines: '1.7K' },
  { name: 'persist', status: 'active', lines: '956' },
  { name: 'backup', status: 'active', lines: '1.3K' },
  { name: 'watchdog', status: 'active', lines: '624' },
  // Interface
  { name: 'api', status: 'processing', lines: '3.4K' },
  { name: 'main', status: 'active', lines: '2.9K' },
]

const statusConfig: Record<ModuleStatus, { color: string; label: string; dotClass: string }> = {
  active: { color: 'var(--accent-green)', label: '运行中', dotClass: 'bg-[var(--accent-green)]' },
  standby: { color: 'var(--text-muted)', label: '待机', dotClass: 'bg-[var(--text-muted)]' },
  processing: { color: 'var(--accent-cyan)', label: '处理中', dotClass: 'bg-[var(--accent-cyan)]' },
}

/* ─────────────────────── ACTIVITY FEED DATA ─────────────────────── */
type ActivityType = 'encode' | 'decode' | 'pulse' | 'dream' | 'node_create' | 'node_delete' | 'regulation' | 'backup'

interface ActivityItem {
  id: number
  type: ActivityType
  description: string
  time: string
}

const activityIcons: Record<ActivityType, { icon: typeof Database; color: string }> = {
  encode: { icon: Database, color: 'var(--dim-z)' },
  decode: { icon: Eye, color: 'var(--dim-x)' },
  pulse: { icon: Zap, color: 'var(--dim-e)' },
  dream: { icon: Moon, color: 'var(--accent-purple)' },
  node_create: { icon: Plus, color: 'var(--accent-green)' },
  node_delete: { icon: Trash2, color: 'var(--accent-red)' },
  regulation: { icon: Activity, color: 'var(--dim-t)' },
  backup: { icon: Save, color: 'var(--dim-s)' },
}

const activityBadgeColors: Record<ActivityType, string> = {
  encode: 'rgba(41,121,255,0.15)',
  decode: 'rgba(0,229,255,0.15)',
  pulse: 'rgba(255,214,0,0.15)',
  dream: 'rgba(213,0,249,0.15)',
  node_create: 'rgba(0,230,118,0.15)',
  node_delete: 'rgba(255,23,68,0.15)',
  regulation: 'rgba(245,0,87,0.15)',
  backup: 'rgba(255,109,0,0.15)',
}

const activityBadgeTextColors: Record<ActivityType, string> = {
  encode: 'var(--dim-z)',
  decode: 'var(--dim-x)',
  pulse: 'var(--dim-e)',
  dream: 'var(--accent-purple)',
  node_create: 'var(--accent-green)',
  node_delete: 'var(--accent-red)',
  regulation: 'var(--dim-t)',
  backup: 'var(--dim-s)',
}

const initialActivities: ActivityItem[] = [
  {
    id: 1,
    type: 'pulse',
    description: '脉冲触发：Type-2 关联型，影响 847 个节点',
    time: '2秒前',
  },
  {
    id: 2,
    type: 'encode',
    description: '记忆编码：7D 向量 [1.0, -2.5, 3.14, ...]，锚点 (100,100,100,0,0,0,0)',
    time: '15秒前',
  },
  {
    id: 3,
    type: 'dream',
    description: '梦境周期完成：3 个阶段，巩固 1,247 条记忆',
    time: '1分钟前',
  },
  {
    id: 4,
    type: 'node_create',
    description: '节点物化：id=4521847，坐标=(200,150,100,0.5,0.3,0.1,0.0)',
    time: '3分钟前',
  },
  {
    id: 5,
    type: 'regulation',
    description: '调节周期：维度压力已归一化，0 个应力违规',
    time: '5分钟前',
  },
]

const activityGenerators: { type: ActivityType; desc: () => string }[] = [
  {
    type: 'encode',
    desc: () => `记忆编码：7D 向量 [${Array.from({ length: 7 }, () => (Math.random() * 10 - 5).toFixed(2)).join(', ')}]，锚点 (${Array.from({ length: 7 }, () => Math.floor(Math.random() * 200)).join(',')})`,
  },
  {
    type: 'pulse',
    desc: () => `脉冲触发：Type-${Math.random() > 0.5 ? '2' : '3'} 关联型，影响 ${Math.floor(Math.random() * 1000 + 100)} 个节点`,
  },
  {
    type: 'dream',
    desc: () => `梦境周期完成：${Math.floor(Math.random() * 3 + 1)} 个阶段，巩固 ${Math.floor(Math.random() * 2000 + 500)} 条记忆`,
  },
  {
    type: 'node_create',
    desc: () => `节点物化：id=${4521848 + Math.floor(Math.random() * 100)}，坐标=(${Array.from({ length: 7 }, () => Math.floor(Math.random() * 200)).join(',')})`,
  },
  {
    type: 'regulation',
    desc: () => `调节周期：维度压力已归一化，${Math.floor(Math.random() * 3)} 个应力违规`,
  },
  {
    type: 'backup',
    desc: () => `备份完成：归档 ${Math.floor(Math.random() * 5000 + 1000)} 条记忆`,
  },
]

/* ─────────────────────── ANIMATION VARIANTS ─────────────────────── */
const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.05, delayChildren: 0.1 },
  },
}

const cardVariants = {
  hidden: { opacity: 0, y: 30 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.5, ease },
  },
}

const moduleVariants = {
  hidden: { opacity: 0, y: 10 },
  visible: (i: number) => ({
    opacity: 1,
    y: 0,
    transition: { delay: i * 0.03, duration: 0.3, ease },
  }),
}

const quickActionVariants = {
  hidden: { opacity: 0, scale: 0.95 },
  visible: (i: number) => ({
    opacity: 1,
    scale: 1,
    transition: { delay: 0.3 + i * 0.1, duration: 0.4, ease },
  }),
}

/* ─────────────────────── MAIN DASHBOARD COMPONENT ─────────────────────── */
export default function Home() {
  const navigate = useNavigate()
  const [activities, setActivities] = useState<ActivityItem[]>(initialActivities)
  const [stats, setStats] = useState<StatsData['data'] | null>(null)
  const [health, setHealth] = useState<HealthData['data'] | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const nextIdRef = useRef(6)

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [statsRes, healthRes] = await Promise.all([api.getStats(), api.getHealth()])
        setStats(statsRes.data)
        setHealth(healthRes.data)
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to connect to TetraMem-XL backend')
      } finally {
        setLoading(false)
      }
    }
    fetchData()
    const interval = setInterval(fetchData, 5000)
    return () => clearInterval(interval)
  }, [])

  const healthMetrics = health ? [
    { name: '守恒状态', value: health.conservation_ok ? 100 : 0, fill: health.conservation_ok ? '#00E676' : '#FF1744' },
    { name: '能量利用率', value: Math.round(health.energy_utilization * 100), fill: health.energy_utilization > 0.9 ? '#FFAB00' : '#00E676' },
    { name: '节点数', value: Math.min(100, Math.round(health.node_count / 100)), fill: '#00E676' },
    { name: '显现率', value: Math.round(health.manifested_ratio * 100), fill: '#00E676' },
    { name: '赫布边数', value: Math.min(100, Math.round(health.hebbian_edge_count / 10)), fill: '#00E676' },
    { name: '赫布权重', value: Math.min(100, Math.round(health.hebbian_avg_weight * 20)), fill: '#00E676' },
    { name: '记忆数', value: Math.min(100, health.memory_count), fill: '#00E676' },
    { name: '前沿规模', value: Math.min(100, health.frontier_size), fill: '#00E676' },
    { name: '健康等级', value: health.level === 'Healthy' ? 100 : health.level === 'Good' ? 80 : 50, fill: health.level === 'Healthy' ? '#00E676' : '#FFAB00' },
    { name: '利用率', value: stats ? Math.round(stats.utilization * 100) : 0, fill: (stats?.utilization ?? 0) > 0.9 ? '#FFAB00' : '#00E676' },
    { name: '物理能量', value: stats ? Math.min(100, Math.round(stats.physical_energy / stats.total_energy * 100)) : 0, fill: '#00E676' },
    { name: '暗能量', value: stats ? Math.min(100, Math.round(stats.dark_energy / stats.total_energy * 100)) : 0, fill: '#00E676' },
  ] : []

  const healthAvg = healthMetrics.length > 0
    ? Math.round(healthMetrics.reduce((s, m) => s + m.value, 0) / healthMetrics.length * 10) / 10
    : 0

  const statsData = stats ? [
    {
      label: '节点总数',
      display: stats.nodes.toLocaleString(),
      icon: Hexagon,
      color: 'var(--dim-x)',
      trend: `${stats.manifested.toLocaleString()} 具现 + ${stats.dark.toLocaleString()} 暗`,
      trendColor: 'var(--accent-green)',
    },
    {
      label: '记忆存储量',
      display: stats.memory_count.toLocaleString(),
      icon: Database,
      color: 'var(--dim-z)',
      trend: `赫布边: ${stats.hebbian_edges}`,
      trendColor: 'var(--dim-z)',
    },
    {
      label: '能量守恒',
      display: stats.conservation_ok ? '100.0%' : 'VIOLATION',
      icon: stats.conservation_ok ? Zap : AlertTriangle,
      color: stats.conservation_ok ? 'var(--dim-e)' : 'var(--accent-red)',
      trend: stats.conservation_ok ? '零损耗已证明' : '检测到违反',
      trendColor: stats.conservation_ok ? 'var(--accent-green)' : 'var(--accent-red)',
      isEnergy: true,
    },
    {
      label: '精度',
      display: '<1e-14',
      icon: Target,
      color: 'var(--dim-mu)',
      trend: '无损编码',
      trendColor: 'var(--accent-purple)',
    },
    {
      label: '利用率',
      display: `${(stats.utilization * 100).toFixed(1)}%`,
      icon: CheckCircle,
      color: 'var(--accent-green)',
      trend: `可用能量: ${stats.available_energy.toFixed(0)}`,
      trendColor: 'var(--text-secondary)',
      isSuccess: true,
    },
  ] : []

  /* Simulate new activity every 5s */
  useEffect(() => {
    const interval = setInterval(() => {
      const r = Math.random()
      let genIndex = 0
      if (r < 0.40) genIndex = 0
      else if (r < 0.65) genIndex = 1
      else if (r < 0.75) genIndex = 2
      else if (r < 0.85) genIndex = 3
      else if (r < 0.95) genIndex = 4
      else genIndex = 5

      const gen = activityGenerators[genIndex]
      const newActivity: ActivityItem = {
        id: nextIdRef.current++,
        type: gen.type,
        description: gen.desc(),
        time: '刚刚',
      }

      setActivities((prev) => {
        const updated = [newActivity, ...prev.map((a, i) => ({
          ...a,
          time: i === 0 ? '2秒前' : i === 1 ? '15秒前' : i === 2 ? '1分钟前' : i === 3 ? '3分钟前' : '5分钟前'
        }))]
        return updated.slice(0, 8)
      })
    }, 5000)

    return () => clearInterval(interval)
  }, [])

  const handleQuickAction = useCallback((path: string) => {
    navigate(path)
  }, [navigate])

  return (
    <div className="relative min-h-[100dvh]">
      {/* 3D Particle Background */}
      <ParticleBackground />

      {/* Content Overlay */}
      <div className="relative z-10 p-6">
        {error && (
          <div className="mx-auto mb-4 max-w-[1440px] rounded-lg border border-[var(--accent-red)] bg-[var(--accent-red)]/10 p-4 text-center">
            <p className="font-body text-sm text-[var(--accent-red)]">{error}</p>
            <p className="mt-1 font-mono text-xs text-[var(--text-muted)]">请确保 TetraMem-XL 后端运行在 127.0.0.1:3456</p>
          </div>
        )}
        <motion.div
          variants={containerVariants}
          initial="hidden"
          animate="visible"
          className="mx-auto max-w-[1440px] space-y-6"
        >
          {/* ─── STAT CARDS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
            {loading ? (
              <div className="glass-card col-span-full p-12 text-center">
                <p className="font-body text-sm text-[var(--text-muted)]">正在连接 TetraMem-XL 后端...</p>
              </div>
            ) : statsData.map((stat) => (
              <motion.div
                key={stat.label}
                variants={cardVariants}
                whileHover={{ y: -4, transition: { duration: 0.25, ease } }}
                className={[
                  'glass-card p-6 transition-shadow duration-250',
                  stat.isEnergy ? 'ring-1' : '',
                ].join(' ')}
                style={stat.isEnergy ? {
                  boxShadow: '0 0 20px rgba(0, 230, 118, 0.2), inset 0 1px 0 rgba(255,255,255,0.05)',
                } : {}}
              >
                {/* Top: icon + label */}
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

                {/* Value */}
                <div
                  className="mb-2 font-mono text-3xl font-bold leading-none tracking-[-0.02em] text-[var(--text-primary)]"
                >
                  {stat.display}
                </div>

                {/* Trend */}
                <div
                  className="font-body text-[13px]"
                  style={{ color: stat.trendColor }}
                >
                  {stat.trend}
                </div>
              </motion.div>
            ))}
          </div>

          {/* ─── SYSTEM HEALTH + MODULE GRID ─── */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-[55%_1fr]">
            {/* 系统健康 Gauge */}
            <motion.div
              variants={cardVariants}
              initial="hidden"
              animate="visible"
              className="glass-panel p-6"
            >
              <div className="mb-4">
                <h2 className="font-display text-2xl font-semibold text-[var(--text-primary)]">
                  系统健康
                </h2>
                <p className="mt-0.5 font-body text-[13px] text-[var(--text-secondary)]">
                  12 维度宇宙监控器
                </p>
              </div>

              <div className="flex flex-col items-center gap-4 sm:flex-row">
                <div className="relative h-[280px] w-[280px] shrink-0">
                  <ResponsiveContainer width="100%" height="100%">
                    <RadialBarChart
                      cx="50%"
                      cy="50%"
                      innerRadius="30%"
                      outerRadius="90%"
                      data={healthMetrics}
                      startAngle={90}
                      endAngle={-270}
                    >
                      <PolarAngleAxis
                        type="number"
                        domain={[0, 100]}
                        dataKey="value"
                        tick={false}
                      />
                      <RadialBar
                        dataKey="value"
                        cornerRadius={4}
                        fill="#00E676"
                        background={{ fill: 'rgba(255,255,255,0.04)' }}
                      />
                    </RadialBarChart>
                  </ResponsiveContainer>
                  <div className="absolute inset-0 flex flex-col items-center justify-center">
                    <span
                      className="font-mono text-4xl font-bold text-[var(--accent-green)]"
                    >
                      {loading ? '...' : `${healthAvg}%`}
                    </span>
                    <span className="mt-1 font-body text-[13px] text-[var(--text-secondary)]">
                      {health?.level ?? 'loading'}
                    </span>
                  </div>
                </div>

                {/* Metric Legend */}
                <div className="grid flex-1 grid-cols-2 gap-x-4 gap-y-2">
                  {healthMetrics.map((m) => (
                    <div key={m.name} className="flex items-center gap-2">
                      <span
                        className="h-2 w-2 rounded-full"
                        style={{ backgroundColor: m.fill }}
                      />
                      <span className="font-body text-[11px] font-semibold uppercase tracking-[0.04em] text-[var(--text-muted)]">
                        {m.name}
                      </span>
                      <span className="ml-auto font-mono text-[11px] text-[var(--text-secondary)]">
                        {m.value}%
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </motion.div>

            {/* 模块状态 Grid */}
            <motion.div
              variants={cardVariants}
              initial="hidden"
              animate="visible"
              className="glass-panel p-6"
            >
              <div className="mb-4">
                <h2 className="font-display text-2xl font-semibold text-[var(--text-primary)]">
                  模块状态
                </h2>
                <p className="mt-0.5 font-body text-[13px] text-[var(--text-secondary)]">
                  19 模块 &middot; 3 层
                </p>
              </div>

              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2">
                {modules.map((mod, i) => {
                  const cfg = statusConfig[mod.status]
                  return (
                    <motion.div
                      key={mod.name}
                      custom={i}
                      variants={moduleVariants}
                      initial="hidden"
                      animate="visible"
                      whileHover={{ x: 2, transition: { duration: 0.15 } }}
                      className="flex h-12 cursor-default items-center gap-3 rounded-lg bg-[var(--bg-surface)] px-3 transition-colors duration-150 hover:bg-[var(--bg-surface-hover)]"
                    >
                      <span
                        className={[
                          'h-2 w-2 shrink-0 rounded-full',
                          mod.status === 'processing' ? 'animate-status-dot-pulse' : '',
                        ].join(' ')}
                        style={{ backgroundColor: cfg.color }}
                      />
                      <span className="flex-1 font-body text-[13px] text-[var(--text-primary)]">
                        {mod.name}
                      </span>
                      <div className="flex items-center gap-2">
                        <span
                          className="rounded px-1.5 py-0.5 font-body text-[10px] font-semibold uppercase tracking-wider"
                          style={{ color: cfg.color, backgroundColor: `${cfg.color}20` }}
                        >
                          {cfg.label}
                        </span>
                        <span className="font-mono text-[10px] text-[var(--text-muted)]">
                          {mod.lines}
                        </span>
                      </div>
                    </motion.div>
                  )
                })}
              </div>
            </motion.div>
          </div>

          {/* ─── RECENT ACTIVITY FEED ─── */}
          <motion.div
            variants={cardVariants}
            initial="hidden"
            animate="visible"
            className="glass-panel p-6"
          >
            <div className="mb-4 flex items-center gap-3">
              <h2 className="font-display text-2xl font-semibold text-[var(--text-primary)]">
                最近活动
              </h2>
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--accent-green)] opacity-75" />
                <span className="relative inline-flex h-2 w-2 rounded-full bg-[var(--accent-green)]" />
              </span>
              <span className="font-body text-[11px] font-semibold uppercase tracking-wider text-[var(--accent-green)]">
                实时
              </span>
            </div>

            <div className="relative">
              {/* Timeline line */}
              <div
                className="absolute bottom-2 left-[71px] top-2 w-[2px] bg-[var(--border-subtle)]"
              />

              <AnimatePresence mode="popLayout">
                {activities.map((activity, i) => {
                  const iconCfg = activityIcons[activity.type]
                  const Icon = iconCfg.icon
                  return (
                    <motion.div
                      key={activity.id}
                      layout
                      initial={{ opacity: 0, y: -30 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, x: 20 }}
                      transition={{ duration: 0.4, ease }}
                      custom={i}
                      className="relative flex items-start gap-4 py-3"
                    >
                      {/* Timestamp */}
                      <div className="w-[60px] shrink-0 pt-1 text-right">
                        <span className="font-mono text-[12px] text-[var(--text-muted)]">
                          {activity.time}
                        </span>
                      </div>

                      {/* Icon */}
                      <div
                        className="relative z-10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full"
                        style={{ backgroundColor: `${iconCfg.color}26` }}
                      >
                        <Icon className="h-4 w-4" style={{ color: iconCfg.color }} />
                      </div>

                      {/* Description */}
                      <div className="flex-1 pt-0.5">
                        <p className="font-body text-[14px] leading-relaxed text-[var(--text-primary)]">
                          {activity.description}
                        </p>
                      </div>

                      {/* Badge */}
                      <div
                        className="shrink-0 rounded px-2 py-1 font-body text-[10px] font-semibold uppercase tracking-wider"
                        style={{
                          backgroundColor: activityBadgeColors[activity.type],
                          color: activityBadgeTextColors[activity.type],
                        }}
                      >
                        {activity.type === 'node_create' ? '节点' :
                          activity.type === 'node_delete' ? '删除' :
                          activity.type}
                      </div>
                    </motion.div>
                  )
                })}
              </AnimatePresence>
            </div>
          </motion.div>

          {/* ─── QUICK ACTION BUTTONS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {[
              {
                icon: Database,
                extraIcon: Plus,
                title: '编码记忆',
                subtitle: '在 7D 空间中存储 1-28D 数据',
                color: 'var(--dim-z)',
                path: '/memory',
              },
              {
                icon: Zap,
                title: '触发脉冲',
                subtitle: '触发 PCNN BFS 脉冲传播',
                color: 'var(--dim-e)',
                path: '/pulse',
              },
              {
                icon: Moon,
                title: '运行梦境',
                subtitle: '重放 \u2192 弱化 \u2192 巩固',
                color: 'var(--accent-purple)',
                path: '/dream',
              },
            ].map((action, i) => (
              <motion.button
                key={action.title}
                custom={i}
                variants={quickActionVariants}
                initial="hidden"
                animate="visible"
                whileHover={{
                  scale: 1.02,
                  borderColor: action.color,
                  backgroundColor: `${action.color}14`,
                  transition: { duration: 0.2 },
                }}
                whileTap={{ scale: 0.98 }}
                onClick={() => handleQuickAction(action.path)}
                className="glass-panel flex items-center gap-4 p-6 text-left transition-colors duration-200"
                style={{ borderColor: 'rgba(255,255,255,0.08)' }}
              >
                <div
                  className="flex h-14 w-14 shrink-0 items-center justify-center rounded-xl"
                  style={{ backgroundColor: `${action.color}1A` }}
                >
                  <action.icon className="h-7 w-7" style={{ color: action.color }} />
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-display text-lg font-semibold text-[var(--text-primary)]">
                      {action.title}
                    </h3>
                    <ChevronRight className="h-4 w-4 text-[var(--text-muted)]" />
                  </div>
                  <p className="mt-0.5 font-body text-[13px] text-[var(--text-secondary)]">
                    {action.subtitle}
                  </p>
                </div>
              </motion.button>
            ))}
          </div>
        </motion.div>

        {/* Bottom spacing */}
        <div className="h-6" />
      </div>
    </div>
  )
}
