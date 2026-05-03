import { useState, useCallback, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Eye,
  Ruler,
  Layers,
  Zap,
  Atom,
  Box,
  ArrowRightLeft,
  Projector,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  api,
  type PhysicsStatusResult,
  type PhysicsProfileResult,
  type PhysicsDistanceResult,
  type PhysicsProjectResult,
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

export default function Physics() {
  const [status, setStatus] = useState<PhysicsStatusResult['data'] | null>(null)
  const [profile, setProfile] = useState<PhysicsProfileResult['data'] | null>(null)

  const [fromX, setFromX] = useState('0')
  const [fromY, setFromY] = useState('0')
  const [fromZ, setFromZ] = useState('0')
  const [toX, setToX] = useState('0')
  const [toY, setToY] = useState('0')
  const [toZ, setToZ] = useState('0')
  const [distanceResult, setDistanceResult] = useState<PhysicsDistanceResult['data'] | null>(null)
  const [distanceLoading, setDistanceLoading] = useState(false)

  const [projCoords, setProjCoords] = useState('0, 0, 0, 0, 0, 0, 0')
  const [projectResult, setProjectResult] = useState<PhysicsProjectResult['data'] | null>(null)
  const [projectLoading, setProjectLoading] = useState(false)

  useEffect(() => {
    api.physicsStatus().then((res) => {
      if (res.success) setStatus(res.data)
    }).catch(() => {})
    api.physicsProfile().then((res) => {
      if (res.success) setProfile(res.data)
    }).catch(() => {})
  }, [])

  const handleCalculateDistance = useCallback(async () => {
    setDistanceLoading(true)
    try {
      const res = await api.physicsDistance(
        [parseFloat(fromX) || 0, parseFloat(fromY) || 0, parseFloat(fromZ) || 0],
        [parseFloat(toX) || 0, parseFloat(toY) || 0, parseFloat(toZ) || 0],
      )
      if (res.success) setDistanceResult(res.data)
    } catch {
      setDistanceResult(null)
    }
    setDistanceLoading(false)
  }, [fromX, fromY, fromZ, toX, toY, toZ])

  const handleProject = useCallback(async () => {
    setProjectLoading(true)
    try {
      const coords = projCoords.split(',').map((s) => parseFloat(s.trim()) || 0)
      const res = await api.physicsProject(coords)
      if (res.success) setProjectResult(res.data)
    } catch {
      setProjectResult(null)
    }
    setProjectLoading(false)
  }, [projCoords])

  const totalNodes = status?.total_nodes ?? 0
  const manifested = status?.manifested_nodes ?? 0
  const dark = status?.dark_nodes ?? 0
  const totalEnergy = status?.total_energy ?? 0
  const engineVersion = status?.physics_engine ?? '—'

  const physicalEnergy = profile?.energy_distribution.physical ?? 0
  const darkEnergy = profile?.energy_distribution.dark ?? 0
  const conservationOk = profile?.conservation_ok ?? false

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
                <Atom className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <h1 className="font-display text-2xl font-bold text-[var(--text-primary)]">
                物理引擎
              </h1>
            </div>
            <p className="font-body text-[13px] text-[var(--text-muted)] max-w-2xl">
              7D 空间物理引擎 — 管理七维空间中的节点分布、能量守恒、距离计算与维度投影。
              物理能量与暗能量共同维持空间的结构稳定。
            </p>
          </motion.div>

          {/* ─── STATUS CARDS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
            {[
              {
                label: '节点总数',
                value: totalNodes.toLocaleString(),
                icon: Layers,
                color: 'var(--accent-cyan)',
              },
              {
                label: '已显化',
                value: manifested.toLocaleString(),
                icon: Eye,
                color: 'var(--accent-green)',
              },
              {
                label: '暗节点',
                value: dark.toLocaleString(),
                icon: Box,
                color: 'var(--text-muted)',
              },
              {
                label: '总能量',
                value: totalEnergy.toLocaleString(),
                icon: Zap,
                color: 'var(--accent-cyan)',
              },
              {
                label: '引擎版本',
                value: engineVersion,
                icon: Atom,
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

          {/* ─── ENERGY PROFILE ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--accent-cyan)26' }}
                >
                  <Zap className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
                </div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  能量分布
                </h2>
              </div>
              <Badge
                variant="outline"
                className="font-mono text-xs"
                style={{
                  borderColor: conservationOk ? 'var(--accent-green)' : 'var(--accent-red)',
                  color: conservationOk ? 'var(--accent-green)' : 'var(--accent-red)',
                }}
              >
                {conservationOk ? '守恒正常' : '守恒异常'}
              </Badge>
            </div>

            <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
              <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                <p className="mb-2 font-body text-[11px] font-semibold uppercase tracking-[0.06em] text-[var(--text-secondary)]">
                  物理能量
                </p>
                <p className="font-mono text-2xl font-bold text-[var(--accent-cyan)]">
                  {physicalEnergy.toLocaleString()}
                </p>
                <div className="mt-3 h-2 w-full overflow-hidden rounded-full bg-[var(--bg-surface)]">
                  <div
                    className="h-full rounded-full transition-all duration-500"
                    style={{
                      width: `${totalEnergy > 0 ? (physicalEnergy / totalEnergy) * 100 : 0}%`,
                      backgroundColor: 'var(--accent-cyan)',
                    }}
                  />
                </div>
              </div>
              <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                <p className="mb-2 font-body text-[11px] font-semibold uppercase tracking-[0.06em] text-[var(--text-secondary)]">
                  暗能量
                </p>
                <p className="font-mono text-2xl font-bold text-[var(--text-muted)]">
                  {darkEnergy.toLocaleString()}
                </p>
                <div className="mt-3 h-2 w-full overflow-hidden rounded-full bg-[var(--bg-surface)]">
                  <div
                    className="h-full rounded-full transition-all duration-500"
                    style={{
                      width: `${totalEnergy > 0 ? (darkEnergy / totalEnergy) * 100 : 0}%`,
                      backgroundColor: 'var(--text-muted)',
                      opacity: 0.6,
                    }}
                  />
                </div>
              </div>
            </div>
          </motion.div>

          {/* ─── DISTANCE CALCULATOR ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-green)26' }}
              >
                <Ruler className="h-5 w-5" style={{ color: 'var(--accent-green)' }} />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                距离计算器
              </h2>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-7">
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  起点 X
                </label>
                <Input
                  value={fromX}
                  onChange={(e) => setFromX(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  起点 Y
                </label>
                <Input
                  value={fromY}
                  onChange={(e) => setFromY(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  起点 Z
                </label>
                <Input
                  value={fromZ}
                  onChange={(e) => setFromZ(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
              <div className="flex items-end justify-center pb-1">
                <ArrowRightLeft className="h-5 w-5 text-[var(--text-muted)]" />
              </div>
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  终点 X
                </label>
                <Input
                  value={toX}
                  onChange={(e) => setToX(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  终点 Y
                </label>
                <Input
                  value={toY}
                  onChange={(e) => setToY(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  终点 Z
                </label>
                <Input
                  value={toZ}
                  onChange={(e) => setToZ(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
            </div>

            <div className="mt-4">
              <Button
                disabled={distanceLoading}
                style={{ backgroundColor: 'var(--accent-green)' }}
                onClick={handleCalculateDistance}
              >
                <Ruler className="mr-1.5 h-4 w-4" />
                {distanceLoading ? '计算中...' : '计算距离'}
              </Button>
            </div>

            {distanceResult && (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-3"
              >
                {[
                  { label: '7D 距离', value: distanceResult.distance_7d.toFixed(4), color: 'var(--accent-cyan)' },
                  { label: '3D 距离', value: distanceResult.distance_3d.toFixed(4), color: 'var(--accent-green)' },
                  { label: '暗维度贡献', value: distanceResult.dark_contribution.toFixed(4), color: 'var(--text-muted)' },
                ].map((item) => (
                  <div
                    key={item.label}
                    className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4 text-center"
                  >
                    <p className="font-body text-[10px] text-[var(--text-muted)]">
                      {item.label}
                    </p>
                    <p className="mt-1 font-mono text-xl font-bold" style={{ color: item.color }}>
                      {item.value}
                    </p>
                  </div>
                ))}
              </motion.div>
            )}
          </motion.div>

          {/* ─── PROJECTION TOOL ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-cyan)26' }}
              >
                <Projector className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                维度投影
              </h2>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <div className="sm:col-span-2">
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  7D 坐标（逗号分隔）
                </label>
                <Input
                  value={projCoords}
                  onChange={(e) => setProjCoords(e.target.value)}
                  className="font-mono text-xs"
                  placeholder="x, y, z, w1, w2, w3, w4"
                />
              </div>
              <div className="flex items-end">
                <Button
                  className="w-full"
                  disabled={projectLoading}
                  style={{ backgroundColor: 'var(--accent-cyan)' }}
                  onClick={handleProject}
                >
                  <Eye className="mr-1.5 h-4 w-4" />
                  {projectLoading ? '投影中...' : '投影'}
                </Button>
              </div>
            </div>

            {projectResult && (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2"
              >
                <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  <p className="mb-2 font-body text-[11px] font-semibold uppercase tracking-[0.06em] text-[var(--text-secondary)]">
                    物理投影
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {projectResult.physical.map((v, i) => (
                      <Badge
                        key={i}
                        variant="outline"
                        className="font-mono text-xs"
                        style={{ borderColor: 'var(--accent-cyan)', color: 'var(--accent-cyan)' }}
                      >
                        D{i + 1}: {v.toFixed(3)}
                      </Badge>
                    ))}
                  </div>
                </div>
                <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  <p className="mb-2 font-body text-[11px] font-semibold uppercase tracking-[0.06em] text-[var(--text-secondary)]">
                    暗维度投影
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {projectResult.dark.map((v, i) => (
                      <Badge
                        key={i}
                        variant="outline"
                        className="font-mono text-xs"
                        style={{ borderColor: 'var(--text-muted)', color: 'var(--text-muted)' }}
                      >
                        D{i + 1}: {v.toFixed(3)}
                      </Badge>
                    ))}
                  </div>
                </div>
              </motion.div>
            )}
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
