import { useState, useCallback, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Server,
  Plus,
  Trash2,
  RefreshCw,
  Crown,
  Shield,
  Zap,
  Clock,
  Users,
  Send,
  Loader2,
  CheckCircle,
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
import { api, type ClusterStatusResult } from '../services/api'

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

interface ProposeLog {
  id: number
  key: string
  value: string
  index: number
  time: string
}

export default function Cluster() {
  const [status, setStatus] = useState<ClusterStatusResult['data'] | null>(null)
  const [loading, setLoading] = useState(false)
  const [initing, setIniting] = useState(false)
  const [error, setError] = useState('')

  const [addId, setAddId] = useState('')
  const [addAddr, setAddAddr] = useState('')
  const [removeId, setRemoveId] = useState('')
  const [adding, setAdding] = useState(false)
  const [removing, setRemoving] = useState(false)

  const [proposeKey, setProposeKey] = useState('')
  const [proposeValue, setProposeValue] = useState('')
  const [proposing, setProposing] = useState(false)
  const [log, setLog] = useState<ProposeLog[]>([])

  const refresh = useCallback(async () => {
    setLoading(true)
    setError('')
    try {
      const res = await api.getClusterStatus()
      if (res.success) setStatus(res.data)
    } catch (err: any) {
      setError(err.message || 'Failed to fetch cluster status')
    }
    setLoading(false)
  }, [])

  useEffect(() => {
    refresh()
    const timer = setInterval(refresh, 5000)
    return () => clearInterval(timer)
  }, [refresh])

  const handleInit = useCallback(async () => {
    setIniting(true)
    setError('')
    try {
      const res = await api.initCluster()
      if (res.success) setStatus(res.data)
    } catch (err: any) {
      setError(err.message || 'Init failed')
    }
    setIniting(false)
  }, [])

  const handleAddNode = useCallback(async () => {
    const nodeId = parseInt(addId, 10)
    if (isNaN(nodeId) || !addAddr.trim()) return
    setAdding(true)
    setError('')
    try {
      await api.addClusterNode(nodeId, addAddr.trim())
      await refresh()
      setAddId('')
      setAddAddr('')
    } catch (err: any) {
      setError(err.message || 'Add node failed')
    }
    setAdding(false)
  }, [addId, addAddr, refresh])

  const handleRemoveNode = useCallback(async () => {
    const nodeId = parseInt(removeId, 10)
    if (isNaN(nodeId)) return
    setRemoving(true)
    setError('')
    try {
      await api.removeClusterNode(nodeId)
      await refresh()
      setRemoveId('')
    } catch (err: any) {
      setError(err.message || 'Remove node failed')
    }
    setRemoving(false)
  }, [removeId, refresh])

  const handlePropose = useCallback(async () => {
    if (!proposeKey.trim() || !proposeValue.trim()) return
    setProposing(true)
    setError('')
    try {
      const res = await api.clusterPropose(proposeKey.trim(), proposeValue.trim())
      if (res.success) {
        setLog((prev) => [
          {
            id: prev.length + 1,
            key: proposeKey.trim(),
            value: proposeValue.trim(),
            index: res.data.log_index,
            time: new Date().toLocaleTimeString(),
          },
          ...prev,
        ])
        setProposeKey('')
        setProposeValue('')
      }
    } catch (err: any) {
      setError(err.message || 'Propose failed')
    }
    setProposing(false)
  }, [proposeKey, proposeValue])

  const roleColor = (role: string) => {
    switch (role) {
      case 'leader': return 'var(--accent-green)'
      case 'follower': return 'var(--accent-cyan)'
      case 'candidate': return 'var(--accent-amber)'
      case 'learner': return 'var(--dim-s)'
      default: return 'var(--text-muted)'
    }
  }

  const roleIcon = (role: string) => {
    switch (role) {
      case 'leader': return Crown
      case 'follower': return Shield
      default: return Server
    }
  }

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
                label: '节点角色',
                value: status ? status.role.toUpperCase() : '--',
                icon: status ? roleIcon(status.role) : Server,
                color: status ? roleColor(status.role) : 'var(--text-muted)',
              },
              {
                label: '当前任期',
                value: status ? String(status.term) : '--',
                icon: Clock,
                color: 'var(--dim-t)',
              },
              {
                label: 'Leader',
                value: status?.leader_id != null ? `Node #${status.leader_id}` : '--',
                icon: Crown,
                color: 'var(--accent-green)',
              },
              {
                label: '已应用日志',
                value: status ? String(status.applied_count) : '--',
                icon: Zap,
                color: 'var(--dim-e)',
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

          {/* ─── ERROR ─── */}
          {error && (
            <div className="flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-4 py-3">
              <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
              <span className="font-body text-[12px] text-[var(--accent-red)]">{error}</span>
            </div>
          )}

          {/* ─── CLUSTER CONTROL ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                集群控制
              </h2>
              <div className="flex gap-2">
                <Button size="sm" variant="outline" disabled={loading} onClick={refresh}>
                  <RefreshCw className={`mr-1.5 h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
                  刷新
                </Button>
                <Button size="sm" disabled={initing} onClick={handleInit}
                  style={{ backgroundColor: 'var(--accent-green)' }}>
                  {initing ? <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" /> : <Zap className="mr-1.5 h-3.5 w-3.5" />}
                  初始化单节点
                </Button>
              </div>
            </div>

            {/* ─── NODE TABLE ─── */}
            <div className="mb-4 rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>节点ID</TableHead>
                    <TableHead>地址</TableHead>
                    <TableHead>角色</TableHead>
                    <TableHead>状态</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!status || status.nodes.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={4} className="py-8 text-center">
                        <Users className="mx-auto mb-2 h-8 w-8 text-[var(--text-muted)] opacity-40" />
                        <p className="font-body text-[12px] text-[var(--text-muted)]">
                          暂无集群节点，点击"初始化单节点"开始
                        </p>
                      </TableCell>
                    </TableRow>
                  ) : (
                    status.nodes.map((node) => {
                      const RIcon = roleIcon(node.role)
                      return (
                        <TableRow key={node.id}>
                          <TableCell className="font-mono text-sm">
                            #{node.id}
                          </TableCell>
                          <TableCell className="font-mono text-xs text-[var(--text-secondary)]">
                            {node.addr}
                          </TableCell>
                          <TableCell>
                            <div className="flex items-center gap-1.5">
                              <RIcon className="h-3.5 w-3.5" style={{ color: roleColor(node.role) }} />
                              <span className="font-body text-[12px] capitalize" style={{ color: roleColor(node.role) }}>
                                {node.role}
                              </span>
                            </div>
                          </TableCell>
                          <TableCell>
                            <Badge
                              variant="outline"
                              className="border-[var(--accent-green)] text-[10px] text-[var(--accent-green)]"
                            >
                              <CheckCircle className="mr-1 h-3 w-3" />
                              在线
                            </Badge>
                          </TableCell>
                        </TableRow>
                      )
                    })
                  )}
                </TableBody>
              </Table>
            </div>

            {/* ─── ADD / REMOVE NODE ─── */}
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <div className="flex items-end gap-2">
                <div className="flex-1">
                  <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">节点ID</label>
                  <input
                    type="number"
                    value={addId}
                    onChange={(e) => setAddId(e.target.value)}
                    placeholder="2"
                    className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none"
                  />
                </div>
                <div className="flex-1">
                  <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">地址</label>
                  <input
                    type="text"
                    value={addAddr}
                    onChange={(e) => setAddAddr(e.target.value)}
                    placeholder="127.0.0.1:3457"
                    className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none"
                  />
                </div>
                <Button size="sm" disabled={adding || !addId || !addAddr} onClick={handleAddNode}
                  style={{ backgroundColor: 'var(--accent-cyan)' }}>
                  {adding ? <Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" /> : <Plus className="mr-1 h-3.5 w-3.5" />}
                  添加
                </Button>
              </div>
              <div className="flex items-end gap-2">
                <div className="flex-1">
                  <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">节点ID</label>
                  <input
                    type="number"
                    value={removeId}
                    onChange={(e) => setRemoveId(e.target.value)}
                    placeholder="2"
                    className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none"
                  />
                </div>
                <Button size="sm" variant="destructive" disabled={removing || !removeId} onClick={handleRemoveNode}>
                  {removing ? <Loader2 className="mr-1 h-3.5 w-3.5 animate-spin" /> : <Trash2 className="mr-1 h-3.5 w-3.5" />}
                  移除
                </Button>
              </div>
            </div>
          </motion.div>

          {/* ─── RAFT PROPOSE ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
              Raft 共识提案
            </h2>
            <div className="flex items-end gap-3">
              <div className="flex-1">
                <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">Key</label>
                <input
                  type="text"
                  value={proposeKey}
                  onChange={(e) => setProposeKey(e.target.value)}
                  placeholder="config.scale_factor"
                  className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none"
                />
              </div>
              <div className="flex-1">
                <label className="mb-1 block font-body text-[11px] text-[var(--text-muted)]">Value</label>
                <input
                  type="text"
                  value={proposeValue}
                  onChange={(e) => setProposeValue(e.target.value)}
                  placeholder="1.5"
                  className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none"
                />
              </div>
              <Button
                disabled={proposing || !proposeKey || !proposeValue}
                onClick={handlePropose}
                style={{ backgroundColor: 'var(--dim-e)' }}
              >
                {proposing ? <Loader2 className="mr-1.5 h-4 w-4 animate-spin" /> : <Send className="mr-1.5 h-4 w-4" />}
                提交提案
              </Button>
            </div>

            {/* ─── PROPOSE LOG ─── */}
            {log.length > 0 && (
              <div className="mt-4 max-h-[240px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Log Index</TableHead>
                      <TableHead>Key</TableHead>
                      <TableHead>Value</TableHead>
                      <TableHead>时间</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {log.map((entry) => (
                      <TableRow key={entry.id}>
                        <TableCell className="font-mono text-xs text-[var(--dim-e)]">
                          #{entry.index}
                        </TableCell>
                        <TableCell className="font-mono text-xs text-[var(--text-primary)]">
                          {entry.key}
                        </TableCell>
                        <TableCell className="font-mono text-xs text-[var(--accent-cyan)]">
                          {entry.value}
                        </TableCell>
                        <TableCell className="flex items-center gap-1 font-body text-[11px] text-[var(--text-muted)]">
                          <Clock className="h-3 w-3" />
                          {entry.time}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            )}
          </motion.div>

          {/* ─── RAFT METRICS ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <h2 className="mb-4 font-display text-xl font-semibold text-[var(--text-primary)]">
              Raft 指标
            </h2>
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
              {[
                { label: '节点ID', value: status ? `#${status.node_id}` : '--' },
                { label: '日志索引', value: status ? String(status.log_index) : '--' },
                { label: '已应用', value: status ? String(status.applied_count) : '--' },
                { label: '集群节点数', value: status ? String(status.nodes.length) : '0' },
              ].map((m) => (
                <div
                  key={m.label}
                  className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4 text-center"
                >
                  <p className="font-body text-[11px] text-[var(--text-muted)]">{m.label}</p>
                  <p className="mt-1 font-mono text-2xl font-bold text-[var(--text-primary)]">{m.value}</p>
                </div>
              ))}
            </div>
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
