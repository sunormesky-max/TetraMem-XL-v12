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
  { method: 'GET', path: '/stats', desc: '宇宙统计信息' },
  { method: 'GET', path: '/health', desc: '健康检查' },
  { method: 'GET', path: '/metrics', desc: 'Prometheus指标' },
  { method: 'POST', path: '/memory/encode', desc: '编码记忆', params: [
    { name: 'anchor', required: true, type: '[x, y, z]' },
    { name: 'data', required: true, type: 'number[]' },
  ] },
  { method: 'POST', path: '/memory/decode', desc: '解码记忆', params: [
    { name: 'anchor', required: true, type: '[x, y, z]' },
    { name: 'data_dim', required: true, type: 'number' },
  ] },
  { method: 'GET', path: '/memory/list', desc: '记忆列表' },
  { method: 'POST', path: '/pulse', desc: '触发脉冲', params: [
    { name: 'source', required: true, type: '[x, y, z]' },
    { name: 'pulse_type', required: true, type: 'string' },
  ] },
  { method: 'POST', path: '/dream', desc: '运行梦境周期' },
  { method: 'POST', path: '/scale', desc: '自动缩放' },
  { method: 'POST', path: '/regulate', desc: '运行调节周期' },
  { method: 'GET', path: '/hebbian/neighbors/{x}/{y}/{z}', desc: '赫布邻居', params: [
    { name: 'x', required: true, type: 'number' },
    { name: 'y', required: true, type: 'number' },
    { name: 'z', required: true, type: 'number' },
  ] },
]

export default function Api() {
  const [selectedEndpoint, setSelectedEndpoint] = useState<Endpoint | null>(null)
  const [requestBody, setRequestBody] = useState('')
  const [responseBody, setResponseBody] = useState('')
  const [history, setHistory] = useState<HistoryItem[]>([])
  const [copied, setCopied] = useState(false)
  const [activeTab, setActiveTab] = useState('explorer')
  const [loading, setLoading] = useState(false)

  const handleSelectEndpoint = useCallback((ep: Endpoint) => {
    setSelectedEndpoint(ep)
    if (ep.method === 'POST' && ep.path.includes('encode')) {
      setRequestBody(JSON.stringify({
        anchor: [128, 128, 128],
        data: [1.0, -2.5, 3.14, 0.0, 2.71],
      }, null, 2))
    } else if (ep.method === 'POST' && ep.path.includes('decode')) {
      setRequestBody(JSON.stringify({
        anchor: [128, 128, 128],
        data_dim: 5,
      }, null, 2))
    } else if (ep.method === 'POST' && ep.path.includes('pulse')) {
      setRequestBody(JSON.stringify({
        source: [128, 128, 128],
        pulse_type: 'associative',
      }, null, 2))
    } else if (ep.method === 'POST' && ep.path.includes('hebbian')) {
      setRequestBody('')
    } else if (ep.method === 'GET' && ep.path.includes('hebbian')) {
      setRequestBody('')
    } else {
      setRequestBody('')
    }
    setResponseBody('')
  }, [])

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
                              <Button variant="ghost" size="sm" className="h-6 px-2 text-[10px]">
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
