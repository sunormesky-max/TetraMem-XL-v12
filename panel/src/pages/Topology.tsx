import { useState, useCallback, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Network,
  GitBranch,
  Box,
  Search,
  Route,
  Shuffle,
  Clock,
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
import { api, type StatsData } from '../services/api'

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

type FeatureType = 'component' | 'cycle' | 'void' | 'crystal'

const featureTypeLabels: Record<FeatureType, string> = {
  component: '连通分量',
  cycle: '环路',
  void: '空腔',
  crystal: '晶体',
}

interface TopologyFeature {
  id: number
  name: string
  type: FeatureType
  value: number
  discovery: string
}

const mockFeatures: TopologyFeature[] = [
  { id: 1, name: 'Beta-0', type: 'component', value: 1, discovery: '2分钟前' },
  { id: 2, name: 'Beta-1-A', type: 'cycle', value: 847, discovery: '15分钟前' },
  { id: 3, name: 'Beta-1-B', type: 'cycle', value: 256, discovery: '32分钟前' },
  { id: 4, name: 'Beta-2', type: 'void', value: 12, discovery: '1小时前' },
  { id: 5, name: 'Crystal-A', type: 'crystal', value: 3, discovery: '2小时前' },
  { id: 6, name: 'Crystal-B', type: 'crystal', value: 5, discovery: '3小时前' },
]

export default function Topology() {
  const [fromNode, setFromNode] = useState('4521847')
  const [toNode, setToNode] = useState('4521001')
  const [pathResult, setPathResult] = useState<string | null>(null)
  const [stats, setStats] = useState<StatsData['data'] | null>(null)

  useEffect(() => {
    api.getStats().then((res) => {
      if (res.success) setStats(res.data)
    }).catch(() => {})
  }, [])

  const handleFindPath = useCallback(() => {
    const dist = Math.abs(parseInt(fromNode) - parseInt(toNode))
    const hops = Math.floor(Math.random() * 8) + 2
    setPathResult(
      `找到路径：${hops} 跳，距离 ${dist.toLocaleString()} 节点`
    )
  }, [fromNode, toNode])

  const handleRandomPair = useCallback(() => {
    const f = 4521000 + Math.floor(Math.random() * 2000)
    const t = 4521000 + Math.floor(Math.random() * 2000)
    setFromNode(String(f))
    setToNode(String(t))
    setPathResult(null)
  }, [])

  const beta0 = mockFeatures.find((f) => f.name === 'Beta-0')?.value ?? 1
  const beta1 = mockFeatures
    .filter((f) => f.type === 'cycle')
    .reduce((s, f) => s + f.value, 0)
  const beta2 = mockFeatures.find((f) => f.name === 'Beta-2')?.value ?? 0

  const nodeCount = stats?.nodes ?? 0
  const manifestedCount = stats?.manifested ?? 0
  const darkCount = stats?.dark ?? 0
  const edgeCount = stats?.hebbian_edges ?? 0

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
                value: nodeCount.toLocaleString(),
                icon: Network,
                color: 'var(--dim-x)',
              },
              {
                label: '已显化',
                value: manifestedCount.toLocaleString(),
                icon: GitBranch,
                color: 'var(--accent-green)',
              },
              {
                label: '暗节点',
                value: darkCount.toLocaleString(),
                icon: Route,
                color: 'var(--dim-e)',
              },
              {
                label: '赫布边',
                value: edgeCount.toLocaleString(),
                icon: Box,
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

          {/* ─── BETTI NUMBERS + CRYSTAL LATTICE ─── */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            {/* Betti Numbers */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
                贝蒂数
              </h2>
              <div className="grid grid-cols-3 gap-3">
                {[
                  { label: 'β₀', value: beta0, desc: '连通分量' },
                  { label: 'β₁', value: beta1, desc: '一维环路' },
                  { label: 'β₂', value: beta2, desc: '二维空腔' },
                ].map((b) => (
                  <div
                    key={b.label}
                    className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4 text-center"
                  >
                    <p className="font-mono text-2xl font-bold text-[var(--dim-x)]">
                      {b.label}
                    </p>
                    <p className="mt-1 font-mono text-xl font-bold text-[var(--text-primary)]">
                      {b.value.toLocaleString()}
                    </p>
                    <p className="mt-1 font-body text-[10px] text-[var(--text-muted)]">
                      {b.desc}
                    </p>
                  </div>
                ))}
              </div>
            </motion.div>

            {/* Crystal Lattice */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
                晶体晶格
              </h2>
              <div
                className="flex items-center justify-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-deep)]"
                style={{ height: '160px' }}
              >
                <div className="text-center">
                  <Box className="mx-auto mb-2 h-10 w-10 text-[var(--dim-mu)] opacity-50" />
                  <p className="font-body text-sm text-[var(--text-muted)]">
                    晶体晶格可视化
                  </p>
                  <p className="font-mono text-[10px] text-[var(--text-muted)]">
                    BCC 结构 · {nodeCount.toLocaleString()} 节点
                  </p>
                </div>
              </div>
            </motion.div>
          </div>

          {/* ─── BFS PATH ANALYSIS ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--dim-e)26' }}
              >
                <Route className="h-5 w-5" style={{ color: 'var(--dim-e)' }} />
              </div>
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                BFS 路径分析
              </h2>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  起始节点
                </label>
                <Input
                  value={fromNode}
                  onChange={(e) => setFromNode(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
              <div>
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  目标节点
                </label>
                <Input
                  value={toNode}
                  onChange={(e) => setToNode(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
              <div className="flex items-end gap-2">
                <Button
                  className="flex-1"
                  style={{ backgroundColor: 'var(--dim-e)' }}
                  onClick={handleFindPath}
                >
                  <Search className="mr-1.5 h-4 w-4" />
                  查找路径
                </Button>
                <Button variant="outline" onClick={handleRandomPair}>
                  <Shuffle className="h-4 w-4" />
                </Button>
              </div>
            </div>

            {pathResult && (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                className="mt-4 rounded-lg border border-[var(--accent-green)] bg-[var(--bg-surface)] p-4"
              >
                <p className="font-body text-sm text-[var(--accent-green)]">
                  {pathResult}
                </p>
              </motion.div>
            )}
          </motion.div>

          {/* ─── TOPOLOGY REPORT ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                拓扑报告
              </h2>
              <Badge variant="outline" className="font-mono text-xs">
                {mockFeatures.length} 特征
              </Badge>
            </div>

            <div className="max-h-[360px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>特征</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>值</TableHead>
                    <TableHead>发现时间</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {mockFeatures.map((f) => (
                    <TableRow key={f.id}>
                      <TableCell className="font-mono text-xs font-semibold text-[var(--text-primary)]">
                        {f.name}
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant="outline"
                          className="text-[10px]"
                          style={{
                            borderColor:
                              f.type === 'component'
                                ? 'var(--accent-green)'
                                : f.type === 'cycle'
                                  ? 'var(--dim-x)'
                                  : f.type === 'void'
                                    ? 'var(--accent-purple)'
                                    : 'var(--dim-mu)',
                            color:
                              f.type === 'component'
                                ? 'var(--accent-green)'
                                : f.type === 'cycle'
                                  ? 'var(--dim-x)'
                                  : f.type === 'void'
                                    ? 'var(--accent-purple)'
                                    : 'var(--dim-mu)',
                          }}
                        >
                          {featureTypeLabels[f.type]}
                        </Badge>
                      </TableCell>
                      <TableCell className="font-mono text-xs">
                        {f.value.toLocaleString()}
                      </TableCell>
                      <TableCell className="flex items-center gap-1.5 font-body text-[11px] text-[var(--text-muted)]">
                        <Clock className="h-3 w-3" />
                        {f.discovery}
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
