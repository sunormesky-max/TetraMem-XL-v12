import { useState, useEffect, useCallback, useMemo } from 'react'
import { motion } from 'framer-motion'
import {
  Hexagon,
  Globe,
  Activity,
  Search,
  Trash2,
  Box,
  Eye,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { api } from '../services/api'
import type { StatsData } from '../services/api'

/* ─────────────── EASING TOKEN ─────────────── */
const ease = [0.16, 1, 0.3, 1] as [number, number, number, number]

/* ─────────────── ANIMATION ─────────────── */
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

/* ─────────────── MOCK NODES FOR TABLE ─────────────── */
interface NodeItem {
  id: number
  coord: number[]
  type: 'physical' | 'dark'
  energy: number
}

function generateMockNodes(count: number): NodeItem[] {
  const nodes: NodeItem[] = []
  for (let i = 0; i < count; i++) {
    const isDark = Math.random() > 0.7
    nodes.push({
      id: 4521000 + i,
      coord: Array.from({ length: 7 }, () =>
        isDark ? Math.random() * 2 - 1 : Math.floor(Math.random() * 256)
      ),
      type: isDark ? 'dark' : 'physical',
      energy: Math.random() * 100,
    })
  }
  return nodes
}

export default function Universe() {
  const [viewMode, setViewMode] = useState<'all' | 'physical' | 'dark'>('all')
  const [autoRotate, setAutoRotate] = useState(true)
  const [search, setSearch] = useState('')
  const [confirmDelete, setConfirmDelete] = useState<number | null>(null)
  const [stats, setStats] = useState<StatsData['data'] | null>(null)

  useEffect(() => {
    let mounted = true
    const fetchStats = async () => {
      try {
        const res = await api.getStats()
        if (mounted && res.success) {
          setStats(res.data)
        }
      } catch {}
    }
    fetchStats()
    const interval = setInterval(fetchStats, 5000)
    return () => {
      mounted = false
      clearInterval(interval)
    }
  }, [])

  const allNodes = useMemo(() => generateMockNodes(50), [])

  const filteredNodes = useMemo(() => {
    return allNodes.filter((n) => {
      const matchMode = viewMode === 'all' || n.type === viewMode
      const matchSearch =
        search === '' || n.id.toString().includes(search)
      return matchMode && matchSearch
    })
  }, [allNodes, viewMode, search])

  const physicalCount = allNodes.filter((n) => n.type === 'physical').length
  const darkCount = allNodes.filter((n) => n.type === 'dark').length

  const manifestedCount = stats?.manifested ?? physicalCount
  const darkCountStats = stats?.dark ?? darkCount
  const totalNodes = stats?.nodes ?? allNodes.length
  const physicalEnergy = stats?.physical_energy ?? 0
  const darkEnergy = stats?.dark_energy ?? 0
  const allocatedEnergy = stats?.allocated_energy ?? 0
  const utilization = stats?.utilization ?? 0

  const handleDelete = useCallback((id: number) => {
    setConfirmDelete(id)
  }, [])

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
                label: '节点总数',
                value: totalNodes.toLocaleString(),
                icon: Hexagon,
                color: 'var(--dim-x)',
              },
              {
                label: 'BCC 晶格',
                value: stats ? (stats.conservation_ok ? 'Active' : 'Unstable') : 'Active',
                icon: Box,
                color: 'var(--accent-green)',
              },
              {
                label: '活跃',
                value: `${manifestedCount.toLocaleString()} / ${darkCountStats.toLocaleString()}`,
                icon: Activity,
                color: 'var(--dim-e)',
              },
              {
                label: '物理 : 暗物质 比例',
                value: totalNodes > 0 ? `${((manifestedCount / totalNodes) * 100).toFixed(1)}%` : '—',
                icon: Globe,
                color: 'var(--dim-mu)',
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

          {/* ─── 3D VIEWER + CONTROLS ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex flex-wrap items-center justify-between gap-4">
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                3D 宇宙查看器
              </h2>
              <div className="flex flex-wrap items-center gap-4">
                <div className="flex items-center gap-2">
                  <Switch
                    checked={autoRotate}
                    onCheckedChange={setAutoRotate}
                  />
                  <span className="font-body text-[13px] text-[var(--text-secondary)]">
                    自动旋转
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="font-body text-[13px] text-[var(--text-secondary)]">
                    视图模式
                  </span>
                  <Select
                    value={viewMode}
                    onValueChange={(v) =>
                      setViewMode(v as 'all' | 'physical' | 'dark')
                    }
                  >
                    <SelectTrigger className="h-8 w-[120px]">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">全部</SelectItem>
                      <SelectItem value="physical">物理</SelectItem>
                      <SelectItem value="dark">暗物质</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>

            {/* Viewer placeholder */}
            <div
              className="flex items-center justify-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-deep)]"
              style={{ height: '360px' }}
            >
              <div className="text-center">
                <Globe className="mx-auto mb-3 h-12 w-12 text-[var(--dim-x)] opacity-40" />
                <p className="font-body text-sm text-[var(--text-muted)]">
                  3D 宇宙查看器 (Three.js)
                </p>
                <p className="mt-1 font-mono text-xs text-[var(--text-muted)]">
                  {autoRotate ? '自动旋转已开启' : '自动旋转已关闭'} · {viewMode === 'all' ? '全部' : viewMode === 'physical' ? '物理' : '暗物质'} 视图
                </p>
              </div>
            </div>

            {/* Node actions */}
            <div className="mt-4 flex flex-wrap gap-3">
              <Button variant="outline" size="sm">
                <Box className="mr-1.5 h-4 w-4" />
                物化节点
              </Button>
              <Button variant="outline" size="sm">
                <Trash2 className="mr-1.5 h-4 w-4" />
                去物化节点
              </Button>
              <div className="flex items-center gap-2">
                <span className="font-body text-[12px] text-[var(--text-muted)]">
                  节点 ID
                </span>
                <Input
                  placeholder="输入节点ID..."
                  className="h-8 w-[160px]"
                />
              </div>
            </div>
          </motion.div>

          {/* ─── ENERGY DISTRIBUTION ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
              能量分布
            </h2>
            <div className="flex h-[180px] items-end gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] p-4">
              {allocatedEnergy > 0 ? (
                <>
                  <div className="flex flex-col items-center gap-2 flex-1">
                    <div
                      className="w-full rounded-t"
                      style={{
                        height: `${Math.max(5, (physicalEnergy / allocatedEnergy) * 100)}%`,
                        background: `linear-gradient(to top, var(--dim-x), var(--dim-e))`,
                        opacity: 0.85,
                      }}
                    />
                    <span className="font-mono text-[10px] text-[var(--text-muted)]">物理</span>
                  </div>
                  <div className="flex flex-col items-center gap-2 flex-1">
                    <div
                      className="w-full rounded-t"
                      style={{
                        height: `${Math.max(5, (darkEnergy / allocatedEnergy) * 100)}%`,
                        background: `linear-gradient(to top, var(--dim-mu), var(--dim-x))`,
                        opacity: 0.85,
                      }}
                    />
                    <span className="font-mono text-[10px] text-[var(--text-muted)]">暗物质</span>
                  </div>
                  <div className="flex flex-col items-center gap-2 flex-1">
                    <div
                      className="w-full rounded-t"
                      style={{
                        height: `${Math.max(5, utilization * 100)}%`,
                        background: `linear-gradient(to top, var(--dim-z), var(--accent-green))`,
                        opacity: 0.85,
                      }}
                    />
                    <span className="font-mono text-[10px] text-[var(--text-muted)]">利用率</span>
                  </div>
                </>
              ) : (
                Array.from({ length: 24 }, (_, i) => {
                  const h = 30 + Math.random() * 70
                  return (
                    <div
                      key={i}
                      className="flex-1 rounded-t"
                      style={{
                        height: `${h}%`,
                        background: `linear-gradient(to top, var(--dim-x), var(--dim-mu))`,
                        opacity: 0.6 + (h / 200),
                      }}
                    />
                  )
                })
              )}
            </div>
          </motion.div>

          {/* ─── NODE REGISTRY ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex flex-wrap items-center justify-between gap-4">
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                节点注册表
              </h2>
              <div className="relative">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--text-muted)]" />
                <Input
                  placeholder="搜索节点..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  className="h-8 w-[240px] pl-9"
                />
              </div>
            </div>

            <p className="mb-3 font-body text-[13px] text-[var(--text-secondary)]">
              显示 {filteredNodes.length} 共 {allNodes.length} 节点
            </p>

            <div className="max-h-[400px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>节点 ID</TableHead>
                    <TableHead>7D 坐标</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>能量</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredNodes.map((node) => (
                    <TableRow key={node.id}>
                      <TableCell className="font-mono text-xs">
                        {node.id}
                      </TableCell>
                      <TableCell className="font-mono text-xs">
                        ({node.coord.map((c) => c.toFixed(1)).join(', ')})
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant={
                            node.type === 'physical'
                              ? 'default'
                              : 'secondary'
                          }
                        >
                          {node.type === 'physical' ? '物理' : '暗物质'}
                        </Badge>
                      </TableCell>
                      <TableCell className="font-mono text-xs">
                        {node.energy.toFixed(2)}
                      </TableCell>
                      <TableCell className="text-right">
                        {confirmDelete === node.id ? (
                          <div className="flex items-center justify-end gap-2">
                            <span className="font-body text-[11px] text-[var(--text-muted)]">
                              确认删除？
                            </span>
                            <Button
                              variant="destructive"
                              size="sm"
                              className="h-6 px-2 text-xs"
                              onClick={() => setConfirmDelete(null)}
                            >
                              删除
                            </Button>
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-6 px-2 text-xs"
                              onClick={() => setConfirmDelete(null)}
                            >
                              取消
                            </Button>
                          </div>
                        ) : (
                          <div className="flex items-center justify-end gap-2">
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-6 px-2 text-xs"
                              onClick={() => handleDelete(node.id)}
                            >
                              <Eye className="mr-1 h-3 w-3" />
                              查看
                            </Button>
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-6 px-2 text-xs text-[var(--accent-red)]"
                              onClick={() => handleDelete(node.id)}
                            >
                              <Trash2 className="mr-1 h-3 w-3" />
                              去物化
                            </Button>
                          </div>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </motion.div>

          {/* ─── COORDINATE PRESETS ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <h2 className="mb-4 font-display text-lg font-semibold text-[var(--text-primary)]">
              7D 坐标预设
            </h2>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
              {[
                { label: '原点', coord: [0, 0, 0, 0, 0, 0, 0] },
                { label: '物理', coord: [128, 128, 128, 0, 0, 0, 0] },
                { label: '暗物质', coord: [0, 0, 0, 0.5, 0.5, 0.5, 0.5] },
                { label: '随机', coord: Array.from({ length: 7 }, () => Math.floor(Math.random() * 256)) },
              ].map((preset) => (
                <Button
                  key={preset.label}
                  variant="outline"
                  className="flex flex-col items-start gap-1 py-3"
                >
                  <span className="font-display text-sm">{preset.label}</span>
                  <span className="font-mono text-[10px] text-[var(--text-muted)]">
                    ({preset.coord.map((c) => (typeof c === 'number' ? c.toFixed(1) : c)).join(', ')})
                  </span>
                </Button>
              ))}
            </div>
            <div className="mt-3 flex flex-wrap gap-2">
              {['X', 'Y', 'Z', 'E', 'S', 'T', '\u03bc'].map((dim) => (
                <span
                  key={dim}
                  className="rounded-full px-3 py-1 font-mono text-[11px] font-semibold"
                  style={{
                    backgroundColor: 'var(--bg-surface)',
                    color: 'var(--text-secondary)',
                  }}
                >
                  {dim}
                </span>
              ))}
            </div>
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
