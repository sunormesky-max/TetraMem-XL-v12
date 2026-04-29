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
  { method: 'GET', path: '/api/v1/universe/nodes', desc: '获取所有节点列表' },
  { method: 'POST', path: '/api/v1/universe/nodes', desc: '创建新节点' },
  { method: 'GET', path: '/api/v1/universe/nodes/:id', desc: '获取指定节点详情' },
  { method: 'DELETE', path: '/api/v1/universe/nodes/:id', desc: '删除指定节点' },
  { method: 'POST', path: '/api/v1/memory/encode', desc: '编码记忆向量到7D晶格' },
  { method: 'POST', path: '/api/v1/memory/decode', desc: '从7D晶格解码记忆向量' },
  { method: 'GET', path: '/api/v1/memory/:id', desc: '获取指定记忆详情' },
  { method: 'POST', path: '/api/v1/pulse/fire', desc: '触发PCNN脉冲传播' },
  { method: 'GET', path: '/api/v1/pulse/history', desc: '获取脉冲历史记录' },
  { method: 'POST', path: '/api/v1/dream/run', desc: '运行梦境周期' },
  { method: 'GET', path: '/api/v1/topology/report', desc: '获取拓扑分析报告' },
]

export default function Api() {
  const [selectedEndpoint, setSelectedEndpoint] = useState<Endpoint | null>(null)
  const [requestBody, setRequestBody] = useState('')
  const [responseBody, setResponseBody] = useState('')
  const [history, setHistory] = useState<HistoryItem[]>([
    { id: 1, method: 'GET', path: '/api/v1/universe/nodes', status: 200, time: '12ms', size: '4.2 KB' },
    { id: 2, method: 'POST', path: '/api/v1/memory/encode', status: 201, time: '45ms', size: '1.1 KB' },
    { id: 3, method: 'GET', path: '/api/v1/pulse/history', status: 200, time: '8ms', size: '2.8 KB' },
  ])
  const [copied, setCopied] = useState(false)
  const [activeTab, setActiveTab] = useState('explorer')

  const handleSelectEndpoint = useCallback((ep: Endpoint) => {
    setSelectedEndpoint(ep)
    // Generate example request body based on endpoint
    if (ep.method === 'POST' && ep.path.includes('encode')) {
      setRequestBody(JSON.stringify({
        coord: [128, 128, 128, 0, 0, 0, 0],
        vector: [1.0, -2.5, 3.14, 0.0, 2.71],
        precision: 14
      }, null, 2))
    } else if (ep.method === 'POST' && ep.path.includes('pulse')) {
      setRequestBody(JSON.stringify({
        type: 'associative',
        origin_node: 4521847,
        energy: 2.4
      }, null, 2))
    } else {
      setRequestBody('')
    }
    setResponseBody('')
  }, [])

  const handleSendRequest = useCallback(() => {
    if (!selectedEndpoint) return
    const mockResponse = {
      status: selectedEndpoint.method === 'POST' ? 201 : 200,
      data: {
        success: true,
        timestamp: new Date().toISOString(),
        endpoint: selectedEndpoint.path,
      }
    }
    setResponseBody(JSON.stringify(mockResponse, null, 2))
    const newItem: HistoryItem = {
      id: history.length + 1,
      method: selectedEndpoint.method,
      path: selectedEndpoint.path,
      status: mockResponse.status,
      time: `${Math.floor(Math.random() * 50 + 5)}ms`,
      size: `${(Math.random() * 5 + 0.5).toFixed(1)} KB`,
    }
    setHistory((prev) => [newItem, ...prev])
  }, [selectedEndpoint, history.length])

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

            {/* ─── ENDPOINT EXPLORER TAB ─── */}
            <TabsContent value="explorer" className="mt-0 space-y-6">
              <div className="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_2fr]">
                {/* Endpoint List */}
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

                {/* Request Builder + Response */}
                <div className="space-y-6">
                  {/* Request Builder */}
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
                        >
                          <Send className="mr-2 h-4 w-4" />
                          发送请求
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

                  {/* Response Viewer */}
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

              {/* ─── REQUEST HISTORY ─── */}
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
                      {history.map((item) => (
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
                                  : 'var(--accent-red)',
                              }}
                            >
                              {item.status}
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
                      ))}
                    </tbody>
                  </table>
                </div>
              </motion.div>
            </TabsContent>

            {/* ─── API DOCS TAB ─── */}
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
