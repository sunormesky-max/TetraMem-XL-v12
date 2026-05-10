import { useState, useCallback } from 'react'
import { motion } from 'framer-motion'
import {
  Send,
  Clipboard,
  CheckCircle,
  Clock,
  Trash2,
  BookOpen,
  Terminal,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'

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

interface Endpoint {
  method: string
  path: string
  desc: string
  params?: { name: string; required: boolean; type: string }[]
}

interface HistoryItem {
  id: number
  method: string
  path: string
  status: number
  time: string
  size: string
}

const endpoints: Endpoint[] = [
  // ── PUBLIC (无需认证) ──
  { method: 'GET', path: '/health', desc: '健康检查' },
  { method: 'POST', path: '/login', desc: 'JWT登录', params: [
    { name: 'username', required: true, type: 'string' },
    { name: 'password', required: true, type: 'string' },
  ] },

  // ── USER (JWT认证) ──
  { method: 'GET', path: '/stats', desc: '宇宙统计信息' },
  { method: 'GET', path: '/metrics', desc: 'Prometheus指标' },
  { method: 'POST', path: '/memory/encode', desc: '编码记忆', params: [
    { name: 'anchor', required: true, type: '[x,y,z]' },
    { name: 'data', required: true, type: 'number[]' },
    { name: 'tags', required: false, type: 'string[]' },
    { name: 'category', required: false, type: 'string' },
    { name: 'description', required: false, type: 'string' },
    { name: 'importance', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/memory/decode', desc: '解码记忆', params: [
    { name: 'anchor', required: true, type: '[x,y,z]' },
    { name: 'data_dim', required: true, type: 'number' },
  ] },
  { method: 'GET', path: '/memory/list', desc: '记忆列表' },
  { method: 'POST', path: '/memory/remember', desc: 'AI记忆存储', params: [
    { name: 'content', required: true, type: 'string' },
    { name: 'tags', required: false, type: 'string[]' },
    { name: 'category', required: false, type: 'string' },
    { name: 'importance', required: false, type: 'number' },
    { name: 'source', required: false, type: 'string' },
  ] },
  { method: 'POST', path: '/memory/recall', desc: 'AI记忆检索', params: [
    { name: 'query', required: true, type: 'string' },
    { name: 'limit', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/memory/associate', desc: '关联发现', params: [
    { name: 'topic', required: true, type: 'string' },
    { name: 'depth', required: false, type: 'number' },
    { name: 'limit', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/memory/forget', desc: '删除记忆', params: [
    { name: 'anchor', required: true, type: '[x,y,z]' },
  ] },
  { method: 'POST', path: '/memory/annotate', desc: '标注记忆', params: [
    { name: 'anchor', required: true, type: '[x,y,z]' },
    { name: 'tags', required: false, type: 'string[]' },
    { name: 'category', required: false, type: 'string' },
    { name: 'description', required: false, type: 'string' },
    { name: 'importance', required: false, type: 'number' },
  ] },
  { method: 'GET', path: '/memory/timeline', desc: '时间轴' },
  { method: 'POST', path: '/memory/trace', desc: '追踪路径', params: [
    { name: 'anchor', required: true, type: '[x,y,z]' },
    { name: 'max_hops', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/pulse', desc: '触发脉冲', params: [
    { name: 'source', required: true, type: '[x,y,z]' },
    { name: 'pulse_type', required: false, type: 'string' },
  ] },
  { method: 'POST', path: '/dream', desc: '运行梦境周期' },
  { method: 'POST', path: '/dream/consolidate', desc: '梦境整合', params: [
    { name: 'importance_threshold', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/context', desc: '上下文管理', params: [
    { name: 'action', required: true, type: '"status"|"reconstruct"|"pre_work"' },
    { name: 'role', required: false, type: 'string' },
    { name: 'content', required: false, type: 'string' },
  ] },
  { method: 'GET', path: '/hebbian/neighbors/{x}/{y}/{z}', desc: '赫布邻居', params: [
    { name: 'x', required: true, type: 'number' },
    { name: 'y', required: true, type: 'number' },
    { name: 'z', required: true, type: 'number' },
  ] },
  { method: 'GET', path: '/dark/query', desc: '查询暗维度节点' },
  { method: 'POST', path: '/dark/flow', desc: '暗能量流动', params: [
    { name: 'from', required: true, type: 'number[]' },
    { name: 'to', required: true, type: 'number[]' },
    { name: 'amount', required: true, type: 'number' },
  ] },
  { method: 'POST', path: '/dark/transfer', desc: '暗能量转移', params: [
    { name: 'source', required: true, type: 'number[]' },
    { name: 'target', required: true, type: 'number[]' },
    { name: 'amount', required: true, type: 'number' },
  ] },
  { method: 'POST', path: '/dark/materialize', desc: '物化节点', params: [
    { name: 'coord', required: true, type: 'number[]' },
    { name: 'energy', required: true, type: 'number' },
    { name: 'physical_ratio', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/dark/dematerialize', desc: '反物化节点', params: [
    { name: 'coord', required: true, type: 'number[]' },
  ] },
  { method: 'GET', path: '/dark/pressure', desc: '暗维度压力' },
  { method: 'GET', path: '/physics/status', desc: '物理引擎状态' },
  { method: 'GET', path: '/physics/profile', desc: '物理引擎配置' },
  { method: 'POST', path: '/physics/distance', desc: '7D距离计算', params: [
    { name: 'from', required: true, type: 'number[]' },
    { name: 'to', required: true, type: 'number[]' },
  ] },
  { method: 'POST', path: '/physics/project', desc: '坐标投影', params: [
    { name: 'coord', required: true, type: 'number[]' },
  ] },
  { method: 'POST', path: '/semantic/search', desc: '语义向量搜索', params: [
    { name: 'data', required: true, type: 'number[]' },
    { name: 'k', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/semantic/query', desc: '语义文本搜索', params: [
    { name: 'text', required: true, type: 'string' },
    { name: 'k', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/semantic/relations', desc: '语义关系', params: [
    { name: 'anchor', required: true, type: '[x,y,z]' },
  ] },
  { method: 'GET', path: '/semantic/status', desc: '语义引擎状态' },
  { method: 'GET', path: '/phase/detect', desc: '相变检测' },
  { method: 'GET', path: '/cluster/status', desc: '集群状态' },
  { method: 'POST', path: '/emotion/pulse', desc: '情绪脉冲', params: [
    { name: 'anchor', required: true, type: '[x,y,z]' },
  ] },
  { method: 'POST', path: '/emotion/dream', desc: '情绪梦境' },
  { method: 'GET', path: '/emotion/status', desc: '情绪状态' },
  { method: 'GET', path: '/perception/status', desc: '感知预算状态' },
  { method: 'GET', path: '/clustering/status', desc: '聚类引擎状态' },
  { method: 'GET', path: '/constitution/status', desc: '宪法状态' },
  { method: 'GET', path: '/events/status', desc: '事件总线状态' },
  { method: 'GET', path: '/watchdog/status', desc: '看门狗状态' },
  { method: 'GET', path: '/agent/observer', desc: '观察者代理' },
  { method: 'GET', path: '/agent/emotion', desc: '情绪代理' },

  // ── ADMIN (JWT + 管理员角色) ──
  { method: 'POST', path: '/scale', desc: '自动缩放' },
  { method: 'POST', path: '/scale/frontier/{max_new}', desc: '前沿扩展', params: [
    { name: 'max_new', required: true, type: 'number' },
  ] },
  { method: 'POST', path: '/regulate', desc: '运行调节周期' },
  { method: 'POST', path: '/backup/create', desc: '创建备份' },
  { method: 'GET', path: '/backup/list', desc: '备份列表' },
  { method: 'POST', path: '/cluster/init', desc: '集群初始化', params: [
    { name: 'node_id', required: false, type: 'number' },
    { name: 'addr', required: false, type: 'string' },
  ] },
  { method: 'POST', path: '/cluster/propose', desc: '集群提案', params: [
    { name: 'key', required: true, type: 'string' },
    { name: 'value', required: true, type: 'string' },
  ] },
  { method: 'POST', path: '/cluster/add-node', desc: '添加集群节点', params: [
    { name: 'node_id', required: true, type: 'number' },
    { name: 'addr', required: true, type: 'string' },
  ] },
  { method: 'POST', path: '/cluster/remove-node', desc: '移除集群节点', params: [
    { name: 'node_id', required: true, type: 'number' },
  ] },
  { method: 'POST', path: '/phase/consensus', desc: '相变共识', params: [
    { name: 'force', required: false, type: 'boolean' },
  ] },
  { method: 'POST', path: '/phase/quorum/start', desc: '启动法定人数', params: [
    { name: 'required_energy_budget', required: false, type: 'number' },
  ] },
  { method: 'POST', path: '/phase/quorum/confirm', desc: '确认法定人数' },
  { method: 'GET', path: '/phase/quorum/status', desc: '法定人数状态' },
  { method: 'POST', path: '/phase/quorum/execute', desc: '执行法定人数决策', params: [
    { name: 'force', required: false, type: 'boolean' },
  ] },
  { method: 'POST', path: '/physics/configure', desc: '配置物理引擎' },
  { method: 'POST', path: '/emotion/crystallize', desc: '情绪结晶' },
  { method: 'POST', path: '/perception/replenish', desc: '感知预算补充' },
  { method: 'POST', path: '/semantic/index-all', desc: '语义全量索引' },
  { method: 'POST', path: '/semantic/extract-concepts', desc: '提取概念' },
  { method: 'POST', path: '/clustering/maintenance', desc: '聚类维护' },
  { method: 'POST', path: '/watchdog/checkup', desc: '看门狗检查' },
  { method: 'POST', path: '/agent/crystal', desc: '晶体代理' },
]

export default function Api() {
  const [selectedEndpoint, setSelectedEndpoint] = useState<Endpoint | null>(null)
  const [requestBody, setRequestBody] = useState('')
  const [responseBody, setResponseBody] = useState('')
  const [history, setHistory] = useState<HistoryItem[]>([])
  const [copied, setCopied] = useState(false)
  const [activeTab, setActiveTab] = useState('explorer')
  const [loading, setLoading] = useState(false)

  const templates: Record<string, object> = {
    '/login': { username: 'admin', password: 'password123' },
    '/memory/encode': { anchor: [128, 128, 128], data: [1.0, -2.5, 3.14, 0.0, 2.71] },
    '/memory/decode': { anchor: [128, 128, 128], data_dim: 5 },
    '/memory/remember': { content: '示例记忆内容', tags: ['示例'], category: 'general', importance: 0.5, source: 'api' },
    '/memory/recall': { query: '示例查询', limit: 10 },
    '/memory/associate': { topic: '示例主题', depth: 2, limit: 5 },
    '/memory/forget': { anchor: [128, 128, 128] },
    '/memory/annotate': { anchor: [128, 128, 128], tags: ['标注'], category: 'annotated', description: '标注描述', importance: 0.7 },
    '/memory/trace': { anchor: [128, 128, 128], max_hops: 3 },
    '/pulse': { source: [128, 128, 128], pulse_type: 'associative' },
    '/dream/consolidate': { importance_threshold: 0.3 },
    '/context': { action: 'status', role: 'user', content: '' },
    '/dark/flow': { from: [100, 100, 100], to: [200, 200, 200], amount: 0.5 },
    '/dark/transfer': { source: [100, 100, 100], target: [200, 200, 200], amount: 0.5 },
    '/dark/materialize': { coord: [128, 128, 128], energy: 1.0, physical_ratio: 0.5 },
    '/dark/dematerialize': { coord: [128, 128, 128] },
    '/physics/distance': { from: [0, 0, 0], to: [128, 128, 128] },
    '/physics/project': { coord: [128, 128, 128] },
    '/semantic/search': { data: [0.1, 0.2, 0.3], k: 5 },
    '/semantic/query': { text: '示例查询', k: 5 },
    '/semantic/relations': { anchor: [128, 128, 128] },
    '/emotion/pulse': { anchor: [128, 128, 128] },
    '/cluster/init': { node_id: 1, addr: 'http://localhost:8080' },
    '/cluster/propose': { key: 'example_key', value: 'example_value' },
    '/cluster/add-node': { node_id: 2, addr: 'http://localhost:8081' },
    '/cluster/remove-node': { node_id: 2 },
    '/phase/consensus': { force: false },
    '/phase/quorum/start': { required_energy_budget: 100 },
    '/phase/quorum/execute': { force: false },
    '/hebbian/neighbors/{x}/{y}/{z}': { x: 128, y: 128, z: 128 },
  }

  const handleSelectEndpoint = useCallback((ep: Endpoint) => {
    setSelectedEndpoint(ep)
    setResponseBody('')
    const template = templates[ep.path]
    if (template) {
      setRequestBody(JSON.stringify(template, null, 2))
    } else {
      setRequestBody('')
    }
  }, [templates])

  const handleSendRequest = useCallback(async () => {
    if (!selectedEndpoint) return
    setLoading(true)
    const startTime = performance.now()
    try {
      const isHebbian = selectedEndpoint.path.includes('/hebbian/neighbors/')
      let fetchPath = `/api${selectedEndpoint.path}`
      if (isHebbian) {
        const parts = requestBody ? JSON.parse(requestBody) : {}
        fetchPath = `/api/hebbian/neighbors/${parts.x ?? 128}/${parts.y ?? 128}/${parts.z ?? 128}`
      }

      const options: RequestInit = {
        method: selectedEndpoint.method,
        headers: { 'Content-Type': 'application/json' },
      }
      if (selectedEndpoint.method === 'POST' && !isHebbian && requestBody) {
        options.body = requestBody
      }

      const res = await fetch(fetchPath, options)
      const elapsed = Math.round(performance.now() - startTime)
      const text = await res.text()
      let formatted = text
      try {
        formatted = JSON.stringify(JSON.parse(text), null, 2)
      } catch {}

      setResponseBody(formatted)

      const newItem: HistoryItem = {
        id: Date.now(),
        method: selectedEndpoint.method,
        path: selectedEndpoint.path,
        status: res.status,
        time: `${elapsed}ms`,
        size: `${(new Blob([text]).size / 1024).toFixed(1)} KB`,
      }
      setHistory((prev) => [newItem, ...prev])
    } catch (err: any) {
      const elapsed = Math.round(performance.now() - startTime)
      setResponseBody(JSON.stringify({ error: err.message }, null, 2))
      const newItem: HistoryItem = {
        id: Date.now(),
        method: selectedEndpoint.method,
        path: selectedEndpoint.path,
        status: 0,
        time: `${elapsed}ms`,
        size: '—',
      }
      setHistory((prev) => [newItem, ...prev])
    } finally {
      setLoading(false)
    }
  }, [selectedEndpoint, requestBody])

  const handleFormatJson = useCallback(() => {
    try {
      const parsed = JSON.parse(requestBody)
      setRequestBody(JSON.stringify(parsed, null, 2))
    } catch { /* ignore */ }
  }, [requestBody])

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(responseBody).catch(() => {})
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }, [responseBody])

  const handleClearHistory = useCallback(() => {
    setHistory([])
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
          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList className="mb-4">
              <TabsTrigger value="explorer">
                <Terminal className="mr-1.5 h-4 w-4" />
                端点浏览器
              </TabsTrigger>
              <TabsTrigger value="docs">
                <BookOpen className="mr-1.5 h-4 w-4" />
                API 文档
              </TabsTrigger>
            </TabsList>

            <TabsContent value="explorer" className="mt-0 space-y-6">
              <div className="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_2fr]">
                <motion.div variants={cardVariants} className="glass-panel p-4">
                  <h2 className="mb-3 font-display text-lg font-semibold text-[var(--text-primary)]">
                    端点浏览器
                  </h2>
                  <div className="max-h-[500px] overflow-auto space-y-1">
                    {endpoints.map((ep) => (
                      <button
                        key={ep.path}
                        onClick={() => handleSelectEndpoint(ep)}
                        className="w-full rounded-lg px-3 py-2.5 text-left transition-colors hover:bg-[var(--bg-surface)]"
                        style={{
                          backgroundColor:
                            selectedEndpoint?.path === ep.path
                              ? 'var(--bg-surface)'
                              : 'transparent',
                          borderLeft: selectedEndpoint?.path === ep.path
                            ? '3px solid var(--dim-x)'
                            : '3px solid transparent',
                        }}
                      >
                        <div className="flex items-center gap-2">
                          <Badge
                            variant="outline"
                            className="h-5 px-1.5 text-[9px]"
                            style={{
                              borderColor:
                                ep.method === 'GET'
                                  ? 'var(--accent-green)'
                                  : ep.method === 'POST'
                                    ? 'var(--dim-z)'
                                    : 'var(--accent-red)',
                              color:
                                ep.method === 'GET'
                                  ? 'var(--accent-green)'
                                  : ep.method === 'POST'
                                    ? 'var(--dim-z)'
                                    : 'var(--accent-red)',
                            }}
                          >
                            {ep.method}
                          </Badge>
                          <span className="font-mono text-[11px] text-[var(--text-primary)]">
                            {ep.path}
                          </span>
                        </div>
                        <p className="mt-0.5 font-body text-[10px] text-[var(--text-muted)]">
                          {ep.desc}
                        </p>
                      </button>
                    ))}
                  </div>
                </motion.div>

                <div className="space-y-6">
                  <motion.div variants={cardVariants} className="glass-panel p-6">
                    <h2 className="mb-4 font-display text-lg font-semibold text-[var(--text-primary)]">
                      请求构建器
                    </h2>

                    {selectedEndpoint ? (
                      <div className="space-y-4">
                        <div className="flex items-center gap-2 rounded-lg bg-[var(--bg-surface)] px-3 py-2">
                          <Badge
                            variant="outline"
                            style={{
                              borderColor:
                                selectedEndpoint.method === 'GET'
                                  ? 'var(--accent-green)'
                                  : selectedEndpoint.method === 'POST'
                                    ? 'var(--dim-z)'
                                    : 'var(--accent-red)',
                              color:
                                selectedEndpoint.method === 'GET'
                                  ? 'var(--accent-green)'
                                  : selectedEndpoint.method === 'POST'
                                    ? 'var(--dim-z)'
                                    : 'var(--accent-red)',
                            }}
                          >
                            {selectedEndpoint.method}
                          </Badge>
                          <span className="font-mono text-sm text-[var(--text-primary)]">
                            {selectedEndpoint.path}
                          </span>
                          <span className="ml-auto font-body text-[11px] text-[var(--text-muted)]">
                            {selectedEndpoint.desc}
                          </span>
                        </div>

                        <div>
                          <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                            参数
                          </label>
                          {selectedEndpoint.params && selectedEndpoint.params.length > 0 ? (
                            <div className="space-y-2">
                              {selectedEndpoint.params.map((p) => (
                                <div key={p.name} className="flex items-center gap-2">
                                  <span className="font-mono text-[11px] text-[var(--text-primary)]">
                                    {p.name}
                                  </span>
                                  <Badge variant="outline" className="text-[9px]">
                                    {p.required ? '必填' : '可选'}
                                  </Badge>
                                  <span className="font-mono text-[10px] text-[var(--text-muted)]">
                                    {p.type}
                                  </span>
                                </div>
                              ))}
                            </div>
                          ) : (
                            <p className="font-body text-[11px] text-[var(--text-muted)]">
                              无参数
                            </p>
                          )}
                        </div>

                        <div>
                          <div className="mb-1 flex items-center justify-between">
                            <label className="font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                              请求体 (JSON)
                            </label>
                            <div className="flex gap-1">
                              <Button variant="ghost" size="sm" className="h-6 px-2 text-[10px]" onClick={handleFormatJson}>
                                格式化 JSON
                              </Button>
                              <Button variant="ghost" size="sm" className="h-6 px-2 text-[10px]" onClick={() => {
                                if (selectedEndpoint) {
                                  const t = templates[selectedEndpoint.path]
                                  if (t) setRequestBody(JSON.stringify(t, null, 2))
                                }
                              }}>
                                加载示例
                              </Button>
                            </div>
                          </div>
                          <textarea
                            value={requestBody}
                            onChange={(e) => setRequestBody(e.target.value)}
                            className="min-h-[120px] w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] p-3 font-mono text-[11px] text-[var(--text-primary)] outline-none"
                            placeholder="{ }"
                          />
                        </div>

                        <Button
                          className="w-full"
                          style={{ backgroundColor: 'var(--accent-green)' }}
                          onClick={handleSendRequest}
                          disabled={loading}
                        >
                          <Send className="mr-2 h-4 w-4" />
                          {loading ? '请求中...' : '发送请求'}
                        </Button>
                      </div>
                    ) : (
                      <div className="flex flex-col items-center justify-center py-12 text-center">
                        <Terminal className="mb-3 h-10 w-10 text-[var(--text-muted)] opacity-50" />
                        <p className="font-body text-sm text-[var(--text-muted)]">
                          点击端点开始探索
                        </p>
                      </div>
                    )}
                  </motion.div>

                  {responseBody && (
                    <motion.div
                      initial={{ opacity: 0, y: 12 }}
                      animate={{ opacity: 1, y: 0 }}
                      className="glass-panel p-6"
                    >
                      <div className="mb-3 flex items-center justify-between">
                        <h2 className="font-display text-lg font-semibold text-[var(--text-primary)]">
                          响应
                        </h2>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-7 px-2"
                          onClick={handleCopy}
                        >
                          {copied ? (
                            <>
                              <CheckCircle className="mr-1.5 h-3.5 w-3.5 text-[var(--accent-green)]" />
                              <span className="text-[var(--accent-green)]">已复制！</span>
                            </>
                          ) : (
                            <>
                              <Clipboard className="mr-1.5 h-3.5 w-3.5" />
                              复制
                            </>
                          )}
                        </Button>
                      </div>
                      <pre className="max-h-[300px] overflow-auto rounded-lg bg-[var(--bg-deep)] p-4 font-mono text-[11px] text-[var(--text-secondary)]">
                        {responseBody}
                      </pre>
                    </motion.div>
                  )}
                </div>
              </div>

              <motion.div variants={cardVariants} className="glass-panel p-6">
                <div className="mb-4 flex items-center justify-between">
                  <h2 className="font-display text-lg font-semibold text-[var(--text-primary)]">
                    请求历史
                  </h2>
                  <Button variant="ghost" size="sm" onClick={handleClearHistory}>
                    <Trash2 className="mr-1.5 h-3.5 w-3.5" />
                    清空
                  </Button>
                </div>

                <div className="max-h-[280px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="border-b border-[var(--border-subtle)]">
                        <th className="px-3 py-2 text-left font-body text-[11px] font-semibold text-[var(--text-muted)]">方法</th>
                        <th className="px-3 py-2 text-left font-body text-[11px] font-semibold text-[var(--text-muted)]">路径</th>
                        <th className="px-3 py-2 text-left font-body text-[11px] font-semibold text-[var(--text-muted)]">状态</th>
                        <th className="px-3 py-2 text-left font-body text-[11px] font-semibold text-[var(--text-muted)]">时间</th>
                        <th className="px-3 py-2 text-left font-body text-[11px] font-semibold text-[var(--text-muted)]">大小</th>
                      </tr>
                    </thead>
                    <tbody>
                      {history.length === 0 ? (
                        <tr>
                          <td colSpan={5} className="px-3 py-6 text-center font-body text-[12px] text-[var(--text-muted)]">
                            暂无请求历史
                          </td>
                        </tr>
                      ) : (
                        history.map((item) => (
                          <tr key={item.id} className="border-b border-[var(--border-subtle)] last:border-0">
                            <td className="px-3 py-2">
                              <Badge variant="outline" className="text-[9px]">
                                {item.method}
                              </Badge>
                            </td>
                            <td className="px-3 py-2 font-mono text-[11px] text-[var(--text-primary)]">
                              {item.path}
                            </td>
                            <td className="px-3 py-2">
                              <span
                                className="font-mono text-[11px]"
                                style={{
                                  color: item.status >= 200 && item.status < 300
                                    ? 'var(--accent-green)'
                                    : item.status === 0
                                      ? 'var(--text-muted)'
                                      : 'var(--accent-red)',
                                }}
                              >
                                {item.status || 'ERR'}
                              </span>
                            </td>
                            <td className="px-3 py-2 flex items-center gap-1 font-mono text-[11px] text-[var(--text-muted)]">
                              <Clock className="h-3 w-3" />
                              {item.time}
                            </td>
                            <td className="px-3 py-2 font-mono text-[11px] text-[var(--text-muted)]">
                              {item.size}
                            </td>
                          </tr>
                        ))
                      )}
                    </tbody>
                  </table>
                </div>
              </motion.div>
            </TabsContent>

            <TabsContent value="docs" className="mt-0">
              <motion.div variants={cardVariants} className="glass-panel p-6">
                <h2 className="mb-6 font-display text-xl font-semibold text-[var(--text-primary)]">
                  API 文档
                </h2>
                <div className="space-y-4">
                  {endpoints.map((ep) => (
                    <div
                      key={ep.path}
                      className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4"
                    >
                      <div className="flex items-center gap-2">
                        <Badge
                          variant="outline"
                          style={{
                            borderColor:
                              ep.method === 'GET'
                                ? 'var(--accent-green)'
                                : ep.method === 'POST'
                                  ? 'var(--dim-z)'
                                  : 'var(--accent-red)',
                            color:
                              ep.method === 'GET'
                                ? 'var(--accent-green)'
                                : ep.method === 'POST'
                                  ? 'var(--dim-z)'
                                  : 'var(--accent-red)',
                          }}
                        >
                          {ep.method}
                        </Badge>
                        <code className="font-mono text-sm text-[var(--text-primary)]">
                          {ep.path}
                        </code>
                      </div>
                      <p className="mt-2 font-body text-[13px] text-[var(--text-secondary)]">
                        {ep.desc}
                      </p>
                      {ep.params && ep.params.length > 0 && (
                        <div className="mt-3">
                          <p className="mb-1 font-body text-[11px] font-semibold text-[var(--text-muted)]">
                            参数
                          </p>
                          <div className="space-y-1">
                            {ep.params.map((p) => (
                              <div key={p.name} className="flex items-center gap-2">
                                <code className="rounded bg-[var(--bg-deep)] px-2 py-0.5 font-mono text-[10px] text-[var(--dim-x)]">
                                  {p.name}
                                </code>
                                <Badge variant="outline" className="text-[8px]">
                                  {p.required ? '必填' : '可选'}
                                </Badge>
                                <span className="font-body text-[10px] text-[var(--text-muted)]">
                                  {p.type}
                                </span>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </motion.div>
            </TabsContent>
          </Tabs>
        </motion.div>
      </div>
    </div>
  )
}
