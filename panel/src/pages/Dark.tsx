import { useState, useCallback, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Eye,
  EyeOff,
  Zap,
  Shield,
  Atom,
  RefreshCw,
  Loader2,
  CheckCircle,
  XCircle,
  ArrowDownToLine,
  ArrowUpFromLine,
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
import {
  api,
  type DarkPressureResult,
  type DarkQueryResult,
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

export default function Dark() {
  const [pressure, setPressure] = useState<DarkPressureResult['data'] | null>(null)
  const [nodes, setNodes] = useState<DarkQueryResult['data'] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  const [coordX, setCoordX] = useState('')
  const [coordY, setCoordY] = useState('')
  const [coordZ, setCoordZ] = useState('')
  const [energy, setEnergy] = useState('')
  const [physicalRatio, setPhysicalRatio] = useState('')
  const [materializing, setMaterializing] = useState(false)
  const [dematerializing, setDematerializing] = useState(false)

  const clearMessages = useCallback(() => {
    setError('')
    setSuccess('')
  }, [])

  const fetchPressure = useCallback(async () => {
    try {
      const res = await api.darkPressure()
      if (res.success) setPressure(res.data)
    } catch {}
  }, [])

  const fetchNodes = useCallback(async () => {
    try {
      const res = await api.darkQuery()
      if (res.success) setNodes(res.data)
    } catch {}
  }, [])

  const refresh = useCallback(async () => {
    setLoading(true)
    clearMessages()
    await Promise.all([fetchPressure(), fetchNodes()])
    setLoading(false)
  }, [fetchPressure, fetchNodes, clearMessages])

  useEffect(() => {
    refresh()
  }, [refresh])

  const handleMaterialize = useCallback(async () => {
    const x = parseFloat(coordX)
    const y = parseFloat(coordY)
    const z = parseFloat(coordZ)
    const e = parseFloat(energy)
    const pr = parseFloat(physicalRatio)
    if ([x, y, z, e, pr].some((v) => isNaN(v))) {
      setError('请填写所有字段并确保为有效数值')
      return
    }
    setMaterializing(true)
    clearMessages()
    try {
      const res = await api.darkMaterialize([x, y, z], e, pr)
      if (res.success) {
        setSuccess(`物质化成功: ${res.data.coord} | energy=${res.data.energy} | ratio=${res.data.physical_ratio}`)
        await refresh()
      }
    } catch (err: any) {
      setError(err.message || '物质化失败')
    }
    setMaterializing(false)
  }, [coordX, coordY, coordZ, energy, physicalRatio, refresh, clearMessages])

  const handleDematerialize = useCallback(async () => {
    const x = parseFloat(coordX)
    const y = parseFloat(coordY)
    const z = parseFloat(coordZ)
    if ([x, y, z].some((v) => isNaN(v))) {
      setError('请填写坐标 (x, y, z) 为有效数值')
      return
    }
    setDematerializing(true)
    clearMessages()
    try {
      const res = await api.darkDematerialize([x, y, z])
      if (res.success) {
        setSuccess(`去物质化成功: ${res.data.coord} | energy=${res.data.energy}`)
        await refresh()
      }
    } catch (err: any) {
      setError(err.message || '去物质化失败')
    }
    setDematerializing(false)
  }, [coordX, coordY, coordZ, refresh, clearMessages])

  const inputClass =
    'w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none'

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
            <div className="mb-1 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-cyan)26' }}
              >
                <Eye className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <h1 className="font-display text-2xl font-bold text-[var(--text-primary)]">
                暗维度
              </h1>
            </div>
            <p className="font-body text-[13px] text-[var(--text-secondary)]">
              7D 暗维度管理面板 — 监控暗能量压力、节点平衡、守恒状态，执行物质化与去物质化操作
            </p>
          </motion.div>

          {/* ─── STAT CARDS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {[
              {
                label: '暗能量',
                value: pressure
                  ? `${pressure.total_dark_energy.toFixed(1)} / ${pressure.total_physical_energy.toFixed(1)}`
                  : '--',
                sub: pressure ? `压力比 ${pressure.pressure_ratio.toFixed(3)}` : '',
                icon: Zap,
                color: 'var(--accent-cyan)',
              },
              {
                label: '节点平衡',
                value: pressure
                  ? pressure.dimension_balance_ok
                    ? '正常'
                    : '失衡'
                  : '--',
                icon: Shield,
                color: pressure?.dimension_balance_ok
                  ? 'var(--accent-green)'
                  : 'var(--accent-red)',
              },
              {
                label: '守恒状态',
                value: pressure
                  ? pressure.pressure_ratio <= 1
                    ? '守恒'
                    : '异常'
                  : '--',
                icon: Atom,
                color:
                  pressure && pressure.pressure_ratio <= 1
                    ? 'var(--accent-green)'
                    : 'var(--accent-cyan)',
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
                {stat.sub && (
                  <p className="mt-2 font-body text-[11px] text-[var(--text-muted)]">
                    {stat.sub}
                  </p>
                )}
              </motion.div>
            ))}
          </div>

          {/* ─── ERROR / SUCCESS ─── */}
          {error && (
            <div className="flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-4 py-3">
              <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
              <span className="font-body text-[12px] text-[var(--accent-red)]">{error}</span>
            </div>
          )}
          {success && (
            <div className="flex items-center gap-2 rounded-lg border border-[var(--accent-green)]/30 bg-[var(--accent-green)]/10 px-4 py-3">
              <CheckCircle className="h-4 w-4 text-[var(--accent-green)]" />
              <span className="font-body text-[12px] text-[var(--accent-green)]">{success}</span>
            </div>
          )}

          {/* ─── MATERIALIZE / DEMATERIALIZE ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--accent-cyan)26' }}
                >
                  <Atom className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
                </div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  物质化 / 去物质化
                </h2>
              </div>
              <Button size="sm" variant="outline" disabled={loading} onClick={refresh}>
                <RefreshCw className={`mr-1.5 h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
                刷新
              </Button>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-5">
              <div>
                <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">X</label>
                <input
                  type="number"
                  value={coordX}
                  onChange={(e) => setCoordX(e.target.value)}
                  placeholder="0"
                  className={inputClass}
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">Y</label>
                <input
                  type="number"
                  value={coordY}
                  onChange={(e) => setCoordY(e.target.value)}
                  placeholder="0"
                  className={inputClass}
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">Z</label>
                <input
                  type="number"
                  value={coordZ}
                  onChange={(e) => setCoordZ(e.target.value)}
                  placeholder="0"
                  className={inputClass}
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">能量</label>
                <input
                  type="number"
                  value={energy}
                  onChange={(e) => setEnergy(e.target.value)}
                  placeholder="1.0"
                  className={inputClass}
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">物理比</label>
                <input
                  type="number"
                  value={physicalRatio}
                  onChange={(e) => setPhysicalRatio(e.target.value)}
                  placeholder="0.5"
                  step="0.01"
                  className={inputClass}
                />
              </div>
            </div>

            <div className="mt-4 flex gap-3">
              <Button
                disabled={materializing}
                onClick={handleMaterialize}
                style={{ backgroundColor: 'var(--accent-cyan)' }}
              >
                {materializing ? (
                  <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                ) : (
                  <ArrowDownToLine className="mr-1.5 h-4 w-4" />
                )}
                物质化
              </Button>
              <Button
                variant="outline"
                disabled={dematerializing}
                onClick={handleDematerialize}
              >
                {dematerializing ? (
                  <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                ) : (
                  <ArrowUpFromLine className="mr-1.5 h-4 w-4" />
                )}
                去物质化
              </Button>
            </div>
          </motion.div>

          {/* ─── DARK NODES TABLE ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-green)26' }}
              >
                <EyeOff className="h-5 w-5" style={{ color: 'var(--accent-green)' }} />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                暗维度节点
              </h2>
              {nodes && (
                <Badge variant="outline" className="text-[10px] border-[var(--accent-cyan)] text-[var(--accent-cyan)]">
                  共 {nodes.total} 个
                </Badge>
              )}
            </div>

            <div className="max-h-[420px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>坐标</TableHead>
                    <TableHead>能量</TableHead>
                    <TableHead>已物质化</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!nodes || nodes.nodes.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={3} className="py-8 text-center">
                        <EyeOff className="mx-auto mb-2 h-8 w-8 text-[var(--text-muted)] opacity-40" />
                        <p className="font-body text-[12px] text-[var(--text-muted)]">
                          暂无暗维度节点数据
                        </p>
                      </TableCell>
                    </TableRow>
                  ) : (
                    nodes.nodes.map((node) => (
                      <TableRow key={String(node.coord)}>
                        <TableCell className="font-mono text-xs text-[var(--text-primary)]">
                          {node.coord}
                        </TableCell>
                        <TableCell className="font-mono text-xs text-[var(--text-secondary)]">
                          {node.energy.toFixed(4)}
                        </TableCell>
                        <TableCell>
                          {node.is_manifested ? (
                            <Badge
                              variant="outline"
                              className="border-[var(--accent-green)] text-[10px] text-[var(--accent-green)]"
                            >
                              <CheckCircle className="mr-1 h-3 w-3" />
                              是
                            </Badge>
                          ) : (
                            <Badge
                              variant="outline"
                              className="border-[var(--text-muted)] text-[10px] text-[var(--text-muted)]"
                            >
                              <XCircle className="mr-1 h-3 w-3" />
                              否
                            </Badge>
                          )}
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
