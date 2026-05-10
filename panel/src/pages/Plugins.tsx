import { useState, useEffect, useCallback } from 'react'
import { motion } from 'framer-motion'
import {
  Puzzle,
  Activity,
  Zap,
  Play,
  Pause,
  Trash2,
  RefreshCw,
  Loader2,
  Battery,
  Cpu,
  Tag,
  ChevronDown,
  ChevronUp,
  AlertCircle,
  CheckCircle,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  api,
  type PluginInfo,
  type PluginStatsResult,
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

const statusConfig: Record<string, { label: string; color: string; bg: string }> = {
  Installed: { label: '已安装', color: 'var(--text-muted)', bg: 'var(--bg-surface)' },
  Enabled: { label: '已启用', color: 'var(--accent-cyan)', bg: 'rgba(0,229,255,0.1)' },
  Running: { label: '运行中', color: 'var(--accent-green)', bg: 'var(--accent-green)26' },
  Disabled: { label: '已禁用', color: 'var(--accent-red)', bg: 'var(--accent-red)26' },
  Error: { label: '错误', color: 'var(--accent-red)', bg: 'var(--accent-red)26' },
  SuspendedEnergyBudgetExceeded: { label: '能量暂停', color: 'var(--accent-amber, #f59e0b)', bg: 'rgba(245,158,11,0.15)' },
}

function formatTime(iso: string | null): string {
  if (!iso) return '从未'
  try {
    return new Date(iso).toLocaleString('zh-CN')
  } catch {
    return iso
  }
}

export default function Plugins() {
  const [plugins, setPlugins] = useState<PluginInfo[]>([])
  const [stats, setStats] = useState<PluginStatsResult['data'] | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [expandedPlugin, setExpandedPlugin] = useState<string | null>(null)
  const [actionLoading, setActionLoading] = useState<Record<string, boolean>>({})

  const fetchData = useCallback(async () => {
    try {
      const [listRes, statsRes] = await Promise.all([
        api.pluginList().catch(() => null),
        api.pluginStats().catch(() => null),
      ])
      if (listRes?.success) setPlugins(listRes.data)
      if (statsRes?.success) setStats(statsRes.data)
      if (!listRes && !statsRes) setError('无法加载插件数据')
      else setError('')
    } catch {
      setError('请求失败')
    }
    setLoading(false)
  }, [])

  useEffect(() => {
    fetchData()
  }, [fetchData])

  const handleAction = useCallback(async (name: string, action: 'enable' | 'disable' | 'uninstall' | 'reset-energy') => {
    setActionLoading((prev) => ({ ...prev, [name]: true }))
    try {
      if (action === 'enable') await api.pluginEnable(name)
      else if (action === 'disable') await api.pluginDisable(name)
      else if (action === 'uninstall') await api.pluginUninstall(name)
      else if (action === 'reset-energy') await api.pluginResetEnergy(name)
      await fetchData()
    } catch {}
    setActionLoading((prev) => ({ ...prev, [name]: false }))
  }, [fetchData])

  const toggleExpand = (name: string) => {
    setExpandedPlugin((prev) => (prev === name ? null : name))
  }

  if (loading) {
    return (
      <div className="flex h-[80dvh] items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-[var(--accent-cyan)]" />
      </div>
    )
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
          {/* ─── HEADER ─── */}
          <motion.div variants={cardVariants}>
            <div className="flex items-center gap-3 mb-2">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-cyan)26' }}
              >
                <Puzzle className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <h1 className="font-display text-2xl font-bold text-[var(--text-primary)]">
                插件市场
              </h1>
            </div>
            <p className="font-body text-[13px] text-[var(--text-muted)]">
              WASM 插件管理 — 安装、启用、禁用与执行沙箱化插件
            </p>
          </motion.div>

          {/* ─── STATS CARDS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {[
              { label: '总插件', value: stats?.total ?? '--', icon: Puzzle, color: 'var(--accent-cyan)' },
              { label: '运行中', value: stats?.running ?? '--', icon: Play, color: 'var(--accent-green)' },
              { label: '已启用', value: stats?.enabled ?? '--', icon: Activity, color: 'var(--accent-cyan)' },
              { label: '已暂停', value: stats?.suspended ?? '--', icon: Pause, color: 'var(--accent-amber, #f59e0b)' },
            ].map((card) => (
              <motion.div key={card.label} variants={cardVariants}>
                <div className="glass-card rounded-xl p-4">
                  <div className="flex items-center justify-between">
                    <span className="font-body text-[11px] uppercase tracking-wider text-[var(--text-muted)]">
                      {card.label}
                    </span>
                    <card.icon className="h-4 w-4" style={{ color: card.color }} />
                  </div>
                  <p className="mt-2 font-display text-2xl font-bold text-[var(--text-primary)]">
                    {card.value}
                  </p>
                </div>
              </motion.div>
            ))}
          </div>

          {/* ─── ENERGY BUDGET ─── */}
          {stats && (
            <motion.div variants={cardVariants}>
              <div className="glass-card rounded-xl p-4">
                <div className="flex items-center gap-3 mb-3">
                  <Battery className="h-4 w-4 text-[var(--accent-green)]" />
                  <span className="font-body text-[12px] uppercase tracking-wider text-[var(--text-muted)]">
                    能量预算
                  </span>
                </div>
                <div className="grid grid-cols-3 gap-4">
                  <div>
                    <p className="font-mono text-[11px] text-[var(--text-muted)]">全局预算</p>
                    <p className="font-display text-lg font-bold text-[var(--text-primary)]">
                      {stats.global_energy_budget.toLocaleString()}
                    </p>
                  </div>
                  <div>
                    <p className="font-mono text-[11px] text-[var(--text-muted)]">已消耗</p>
                    <p className="font-display text-lg font-bold text-[var(--accent-cyan)]">
                      {stats.total_energy_consumed.toLocaleString()}
                    </p>
                  </div>
                  <div>
                    <p className="font-mono text-[11px] text-[var(--text-muted)]">总执行次数</p>
                    <p className="font-display text-lg font-bold text-[var(--text-primary)]">
                      {stats.total_executions.toLocaleString()}
                    </p>
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {/* ─── REFRESH ─── */}
          <motion.div variants={cardVariants} className="flex justify-end">
            <Button
              variant="ghost"
              size="sm"
              onClick={fetchData}
              className="text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            >
              <RefreshCw className="mr-2 h-3 w-3" />
              刷新
            </Button>
          </motion.div>

          {/* ─── ERROR ─── */}
          {error && (
            <motion.div variants={cardVariants}>
              <div className="flex items-center gap-2 rounded-lg bg-[var(--accent-red)26] px-4 py-3">
                <AlertCircle className="h-4 w-4 text-[var(--accent-red)]" />
                <span className="font-body text-[13px] text-[var(--accent-red)]">{error}</span>
              </div>
            </motion.div>
          )}

          {/* ─── PLUGIN LIST ─── */}
          {plugins.length === 0 && !error ? (
            <motion.div variants={cardVariants}>
              <div className="glass-card rounded-xl p-12 text-center">
                <Puzzle className="mx-auto h-12 w-12 text-[var(--text-muted)] opacity-30" />
                <p className="mt-4 font-body text-[14px] text-[var(--text-muted)]">
                  暂无已安装的插件
                </p>
                <p className="mt-1 font-mono text-[11px] text-[var(--text-muted)]">
                  通过 API 端点 <code className="text-[var(--accent-cyan)]">POST /api/plugins/install</code> 安装 WASM 插件
                </p>
              </div>
            </motion.div>
          ) : (
            <div className="space-y-3">
              {plugins.map((plugin) => {
                const sc = statusConfig[plugin.status] ?? statusConfig.Installed
                const isExpanded = expandedPlugin === plugin.manifest.name
                const isLoading = actionLoading[plugin.manifest.name] ?? false

                return (
                  <motion.div key={plugin.manifest.name} variants={cardVariants}>
                    <div className="glass-card rounded-xl overflow-hidden">
                      {/* ─── Plugin Header ─── */}
                      <button
                        onClick={() => toggleExpand(plugin.manifest.name)}
                        className="flex w-full items-center justify-between px-5 py-4 text-left hover:bg-[var(--bg-surface-hover)] transition-colors"
                      >
                        <div className="flex items-center gap-4">
                          <div
                            className="flex h-9 w-9 items-center justify-center rounded-lg"
                            style={{ backgroundColor: sc.bg }}
                          >
                            <Cpu className="h-4 w-4" style={{ color: sc.color }} />
                          </div>
                          <div>
                            <div className="flex items-center gap-2">
                              <span className="font-display text-[14px] font-semibold text-[var(--text-primary)]">
                                {plugin.manifest.name}
                              </span>
                              <span className="font-mono text-[11px] text-[var(--text-muted)]">
                                v{plugin.manifest.version}
                              </span>
                              <Badge
                                variant="outline"
                                className="text-[10px] font-mono"
                                style={{
                                  color: sc.color,
                                  borderColor: sc.color,
                                  backgroundColor: sc.bg,
                                }}
                              >
                                {sc.label}
                              </Badge>
                            </div>
                            <p className="mt-0.5 font-body text-[12px] text-[var(--text-muted)] line-clamp-1">
                              {plugin.manifest.description}
                            </p>
                          </div>
                        </div>
                        <div className="flex items-center gap-3">
                          <div className="hidden sm:flex items-center gap-4 text-[11px] font-mono text-[var(--text-muted)]">
                            <span>{plugin.executions} 次执行</span>
                            <span>{plugin.energy_consumed} 能量</span>
                          </div>
                          {isExpanded ? (
                            <ChevronUp className="h-4 w-4 text-[var(--text-muted)]" />
                          ) : (
                            <ChevronDown className="h-4 w-4 text-[var(--text-muted)]" />
                          )}
                        </div>
                      </button>

                      {/* ─── Expanded Details ─── */}
                      {isExpanded && (
                        <div className="border-t border-[var(--border-subtle)] px-5 py-4">
                          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                            {/* Left: Metadata */}
                            <div className="space-y-3">
                              <div>
                                <p className="font-mono text-[10px] uppercase tracking-wider text-[var(--text-muted)] mb-1">
                                  元数据
                                </p>
                                <div className="space-y-1 text-[12px]">
                                  <div className="flex justify-between">
                                    <span className="text-[var(--text-muted)]">作者</span>
                                    <span className="text-[var(--text-primary)]">{plugin.manifest.author}</span>
                                  </div>
                                  <div className="flex justify-between">
                                    <span className="text-[var(--text-muted)]">API 版本</span>
                                    <span className="text-[var(--text-primary)]">{plugin.manifest.api_version}</span>
                                  </div>
                                  <div className="flex justify-between">
                                    <span className="text-[var(--text-muted)]">能量预算</span>
                                    <span className="text-[var(--text-primary)]">{plugin.manifest.energy_budget.toLocaleString()}</span>
                                  </div>
                                  <div className="flex justify-between">
                                    <span className="text-[var(--text-muted)]">安装时间</span>
                                    <span className="text-[var(--text-primary)]">{formatTime(plugin.installed_at)}</span>
                                  </div>
                                  <div className="flex justify-between">
                                    <span className="text-[var(--text-muted)]">上次执行</span>
                                    <span className="text-[var(--text-primary)]">{formatTime(plugin.last_execution)}</span>
                                  </div>
                                </div>
                              </div>

                              {/* Tags */}
                              {plugin.manifest.tags.length > 0 && (
                                <div>
                                  <p className="font-mono text-[10px] uppercase tracking-wider text-[var(--text-muted)] mb-1">
                                    标签
                                  </p>
                                  <div className="flex flex-wrap gap-1">
                                    {plugin.manifest.tags.map((tag) => (
                                      <Badge
                                        key={tag}
                                        variant="outline"
                                        className="text-[10px] font-mono text-[var(--accent-cyan)] border-[var(--accent-cyan)]/30"
                                      >
                                        <Tag className="mr-1 h-2.5 w-2.5" />
                                        {tag}
                                      </Badge>
                                    ))}
                                  </div>
                                </div>
                              )}

                              {/* Permissions */}
                              <div>
                                <p className="font-mono text-[10px] uppercase tracking-wider text-[var(--text-muted)] mb-1">
                                  权限
                                </p>
                                <div className="grid grid-cols-2 gap-1">
                                  {Object.entries(plugin.manifest.permissions).map(([key, val]) => (
                                    <div key={key} className="flex items-center gap-1.5 text-[11px]">
                                      {val ? (
                                        <CheckCircle className="h-3 w-3 text-[var(--accent-green)]" />
                                      ) : (
                                        <div className="h-3 w-3 rounded-full border border-[var(--text-muted)]/30" />
                                      )}
                                      <span className={val ? 'text-[var(--text-primary)]' : 'text-[var(--text-muted)]'}>
                                        {key.replace(/_/g, ' ')}
                                      </span>
                                    </div>
                                  ))}
                                </div>
                              </div>
                            </div>

                            {/* Right: Actions */}
                            <div className="space-y-3">
                              <p className="font-mono text-[10px] uppercase tracking-wider text-[var(--text-muted)]">
                                操作
                              </p>
                              <div className="flex flex-wrap gap-2">
                                {plugin.status === 'Enabled' || plugin.status === 'Running' ? (
                                  <Button
                                    variant="outline"
                                    size="sm"
                                    disabled={isLoading}
                                    onClick={() => handleAction(plugin.manifest.name, 'disable')}
                                    className="border-[var(--accent-amber, #f59e0b)]/50 text-[var(--accent-amber, #f59e0b)] hover:bg-[var(--accent-amber, #f59e0b)]/10"
                                  >
                                    {isLoading ? (
                                      <Loader2 className="mr-1.5 h-3 w-3 animate-spin" />
                                    ) : (
                                      <Pause className="mr-1.5 h-3 w-3" />
                                    )}
                                    禁用
                                  </Button>
                                ) : (
                                  <Button
                                    variant="outline"
                                    size="sm"
                                    disabled={isLoading}
                                    onClick={() => handleAction(plugin.manifest.name, 'enable')}
                                    className="border-[var(--accent-green)]/50 text-[var(--accent-green)] hover:bg-[var(--accent-green)]/10"
                                  >
                                    {isLoading ? (
                                      <Loader2 className="mr-1.5 h-3 w-3 animate-spin" />
                                    ) : (
                                      <Play className="mr-1.5 h-3 w-3" />
                                    )}
                                    启用
                                  </Button>
                                )}
                                <Button
                                  variant="outline"
                                  size="sm"
                                  disabled={isLoading}
                                  onClick={() => handleAction(plugin.manifest.name, 'reset-energy')}
                                  className="border-[var(--accent-cyan)]/50 text-[var(--accent-cyan)] hover:bg-[var(--accent-cyan)]/10"
                                >
                                  {isLoading ? (
                                    <Loader2 className="mr-1.5 h-3 w-3 animate-spin" />
                                  ) : (
                                    <Zap className="mr-1.5 h-3 w-3" />
                                  )}
                                  重置能量
                                </Button>
                                <Button
                                  variant="outline"
                                  size="sm"
                                  disabled={isLoading}
                                  onClick={() => handleAction(plugin.manifest.name, 'uninstall')}
                                  className="border-[var(--accent-red)]/50 text-[var(--accent-red)] hover:bg-[var(--accent-red)]/10"
                                >
                                  {isLoading ? (
                                    <Loader2 className="mr-1.5 h-3 w-3 animate-spin" />
                                  ) : (
                                    <Trash2 className="mr-1.5 h-3 w-3" />
                                  )}
                                  卸载
                                </Button>
                              </div>

                              {/* Usage Stats */}
                              <div className="mt-4 glass-card rounded-lg p-3">
                                <p className="font-mono text-[10px] uppercase tracking-wider text-[var(--text-muted)] mb-2">
                                  使用统计
                                </p>
                                <div className="grid grid-cols-2 gap-3 text-[12px]">
                                  <div>
                                    <span className="text-[var(--text-muted)]">总执行</span>
                                    <p className="font-display font-bold text-[var(--text-primary)]">
                                      {plugin.executions.toLocaleString()}
                                    </p>
                                  </div>
                                  <div>
                                    <span className="text-[var(--text-muted)]">能量消耗</span>
                                    <p className="font-display font-bold text-[var(--accent-cyan)]">
                                      {plugin.energy_consumed.toLocaleString()}
                                    </p>
                                  </div>
                                  <div>
                                    <span className="text-[var(--text-muted)]">能量预算</span>
                                    <p className="font-display font-bold text-[var(--text-primary)]">
                                      {plugin.manifest.energy_budget.toLocaleString()}
                                    </p>
                                  </div>
                                  <div>
                                    <span className="text-[var(--text-muted)]">利用率</span>
                                    <p className="font-display font-bold text-[var(--text-primary)]">
                                      {plugin.manifest.energy_budget > 0
                                        ? ((plugin.energy_consumed / plugin.manifest.energy_budget) * 100).toFixed(1)
                                        : 0}%
                                    </p>
                                  </div>
                                </div>
                              </div>
                            </div>
                          </div>
                        </div>
                      )}
                    </div>
                  </motion.div>
                )
              })}
            </div>
          )}

          {/* ─── API INFO ─── */}
          <motion.div variants={cardVariants}>
            <div className="glass-card rounded-xl p-4">
              <h3 className="font-display text-[13px] font-semibold text-[var(--text-primary)] mb-2">
                API 端点
              </h3>
              <div className="space-y-1 font-mono text-[11px] text-[var(--text-muted)]">
                <div><span className="text-[var(--accent-cyan)]">GET</span> /api/plugins/list — 列出所有插件</div>
                <div><span className="text-[var(--accent-cyan)]">GET</span> /api/plugins/stats — 插件统计</div>
                <div><span className="text-[var(--accent-green)]">POST</span> /api/plugins/install — 安装插件（Admin）</div>
                <div><span className="text-[var(--accent-green)]">POST</span> /api/plugins/:name/enable — 启用（Admin）</div>
                <div><span className="text-[var(--accent-green)]">POST</span> /api/plugins/:name/disable — 禁用（Admin）</div>
                <div><span className="text-[var(--accent-green)]">POST</span> /api/plugins/:name/execute — 执行（Admin）</div>
                <div><span className="text-[var(--accent-green)]">POST</span> /api/plugins/:name/uninstall — 卸载（Admin）</div>
                <div><span className="text-[var(--accent-green)]">POST</span> /api/plugins/:name/reset-energy — 重置能量（Admin）</div>
              </div>
            </div>
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
