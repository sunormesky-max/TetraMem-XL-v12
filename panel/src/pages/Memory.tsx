import { useState, useMemo, useEffect, useCallback } from 'react'
import { motion } from 'framer-motion'
import {
  Database,
  Plus,
  Eye,
  Trash2,
  CheckCircle,
  XCircle,
  Copy,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Slider } from '@/components/ui/slider'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { api } from '../services/api'
import type { EncodeResult, DecodeResult, MemoryListItem } from '../services/api'

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

interface MemoryItem {
  id: string
  coord: number[]
  dims: number
  raw: string
}

function memoryItemFromApi(m: MemoryListItem): MemoryItem {
  const coordMatch = m.anchor.match(/\(\s*(-?[\d.]+)\s*,\s*(-?[\d.]+)\s*,\s*(-?[\d.]+)/)
  return {
    id: m.anchor,
    coord: coordMatch
      ? [Math.round(+coordMatch[1]), Math.round(+coordMatch[2]), Math.round(+coordMatch[3])]
      : [0, 0, 0],
    dims: m.data_dim,
    raw: m.description || m.anchor,
  }
}

export default function Memory() {
  const [memories, setMemories] = useState<MemoryItem[]>([])
  const [selectedId, setSelectedId] = useState<string>('')
  const [encodeCoord, setEncodeCoord] = useState('100,100,100')
  const [encodeVector, setEncodeVector] = useState('[1.0, -2.5, 3.14]')
  const [precision, setPrecision] = useState([14])
  const [showDetail, setShowDetail] = useState<MemoryItem | null>(null)
  const [copied, setCopied] = useState(false)

  const [encodeLoading, setEncodeLoading] = useState(false)
  const [encodeResult, setEncodeResult] = useState<EncodeResult['data'] | null>(null)
  const [encodeError, setEncodeError] = useState('')

  const [decodeLoading, setDecodeLoading] = useState(false)
  const [decodeResult, setDecodeResult] = useState<DecodeResult['data'] | null>(null)
  const [decodeError, setDecodeError] = useState('')

  const [listError, setListError] = useState('')

  const loadMemories = useCallback(async () => {
    try {
      setListError('')
      const res = await api.listMemories()
      const items = res.data.map((m) => memoryItemFromApi(m))
      setMemories(items)
    } catch (err: any) {
      setListError(err.message || '加载记忆列表失败')
    }
  }, [])

  useEffect(() => {
    loadMemories()
  }, [loadMemories])

  const selected = useMemo(
    () => memories.find((m) => m.id === selectedId) || null,
    [memories, selectedId]
  )

  const handleCopy = (id: string) => {
    navigator.clipboard.writeText(id).catch(() => {})
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  const handleEncode = async () => {
    setEncodeLoading(true)
    setEncodeError('')
    setEncodeResult(null)
    try {
      const anchorParts = encodeCoord.split(',').map(Number)
      if (anchorParts.length !== 3 || anchorParts.some(isNaN)) {
        throw new Error('锚点坐标必须是3个数字，用逗号分隔')
      }
      const vector = JSON.parse(encodeVector)
      if (!Array.isArray(vector) || vector.some((v: any) => typeof v !== 'number')) {
        throw new Error('数据向量必须是JSON数字数组')
      }
      const result = await api.encodeMemory(
        anchorParts as [number, number, number],
        vector as number[]
      )
      setEncodeResult(result.data)
      loadMemories()
    } catch (err: any) {
      setEncodeError(err.message || '编码失败')
    } finally {
      setEncodeLoading(false)
    }
  }

  const handleDecode = async () => {
    if (!selected) return
    if (!selected.dims || selected.dims === 0) {
      setDecodeError('无法获取维度数，请在记忆库中选择一条有效记忆')
      return
    }
    setDecodeLoading(true)
    setDecodeError('')
    setDecodeResult(null)
    try {
      const anchor = selected.coord as [number, number, number]
      const result = await api.decodeMemory(anchor, selected.dims)
      if (result.data) {
        setDecodeResult(result.data)
      } else {
        setDecodeError('解码失败：未找到对应记忆数据')
      }
    } catch (err: any) {
      setDecodeError(err.message || '解码失败')
    } finally {
      setDecodeLoading(false)
    }
  }

  const handleDelete = async (item: MemoryItem) => {
    try {
      const anchor = item.coord as [number, number, number]
      await api.forget(anchor)
      if (showDetail?.id === item.id) setShowDetail(null)
      if (selectedId === item.id) {
        setSelectedId('')
        setDecodeResult(null)
      }
      loadMemories()
    } catch (err: any) {
      setListError(err.message || '删除失败')
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
          {/* ─── ENCODE / DECODE CARDS ─── */}
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            {/* Encode Memory */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <div className="mb-4 flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--dim-z)26' }}
                >
                  <Plus className="h-5 w-5" style={{ color: 'var(--dim-z)' }} />
                </div>
                <div>
                  <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                    编码记忆
                  </h2>
                  <p className="font-body text-[12px] text-[var(--text-muted)]">
                    将高维向量存储到 7D 晶格中
                  </p>
                </div>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                    锚点坐标
                  </label>
                  <Input
                    value={encodeCoord}
                    onChange={(e) => setEncodeCoord(e.target.value)}
                    className="font-mono text-xs"
                  />
                </div>
                <div>
                  <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                    数据向量
                  </label>
                  <Input
                    value={encodeVector}
                    onChange={(e) => setEncodeVector(e.target.value)}
                    className="font-mono text-xs"
                  />
                </div>
                <div>
                  <label className="mb-2 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                    精度 (1e-{precision[0]})
                  </label>
                  <Slider
                    value={precision}
                    onValueChange={setPrecision}
                    min={1}
                    max={15}
                    step={1}
                  />
                </div>
                <div className="grid grid-cols-3 gap-3">
                  <div className="rounded-lg bg-[var(--bg-surface)] p-3 text-center">
                    <p className="font-body text-[10px] text-[var(--text-muted)]">
                      维度数
                    </p>
                    <p className="font-mono text-lg font-bold text-[var(--text-primary)]">
                      3
                    </p>
                  </div>
                  <div className="rounded-lg bg-[var(--bg-surface)] p-3 text-center">
                    <p className="font-body text-[10px] text-[var(--text-muted)]">
                      精度
                    </p>
                    <p className="font-mono text-lg font-bold text-[var(--dim-mu)]">
                      1e-{precision[0]}
                    </p>
                  </div>
                  <div className="rounded-lg bg-[var(--bg-surface)] p-3 text-center">
                    <p className="font-body text-[10px] text-[var(--text-muted)]">
                      能量消耗
                    </p>
                    <p className="font-mono text-lg font-bold text-[var(--dim-e)]">
                      2.4
                    </p>
                  </div>
                </div>
                {encodeError && (
                  <div className="flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                    <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
                    <span className="font-body text-[12px] text-[var(--accent-red)]">
                      {encodeError}
                    </span>
                  </div>
                )}
                <Button
                  className="w-full"
                  style={{ backgroundColor: 'var(--dim-z)' }}
                  onClick={handleEncode}
                  disabled={encodeLoading}
                >
                  <Database className="mr-2 h-4 w-4" />
                  {encodeLoading ? '编码中...' : '编码记忆'}
                </Button>
              </div>

              {/* Encoding Result */}
              <div className="mt-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                <h3 className="mb-2 font-display text-sm font-semibold text-[var(--text-primary)]">
                  编码结果
                </h3>
                {encodeResult ? (
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        锚点
                      </span>
                      <code className="flex-1 rounded bg-[var(--bg-deep)] px-3 py-2 font-mono text-[11px] text-[var(--accent-green)]">
                        {encodeResult.anchor}
                      </code>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-8 px-2"
                        onClick={() => handleCopy(encodeResult.anchor)}
                      >
                        {copied ? (
                          <CheckCircle className="h-4 w-4 text-[var(--accent-green)]" />
                        ) : (
                          <Copy className="h-4 w-4" />
                        )}
                      </Button>
                    </div>
                    <div className="flex items-center gap-3">
                      <div className="flex items-center gap-1">
                        <span className="font-body text-[10px] text-[var(--text-muted)]">
                          数据维度
                        </span>
                        <Badge variant="outline" className="font-mono text-[10px]">
                          {encodeResult.data_dim}D
                        </Badge>
                      </div>
                      <div className="flex items-center gap-1">
                        <span className="font-body text-[10px] text-[var(--text-muted)]">
                          显化
                        </span>
                        {encodeResult.manifested ? (
                          <CheckCircle className="h-3.5 w-3.5 text-[var(--accent-green)]" />
                        ) : (
                          <XCircle className="h-3.5 w-3.5 text-[var(--text-muted)]" />
                        )}
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="flex items-center gap-2">
                    <code className="flex-1 rounded bg-[var(--bg-deep)] px-3 py-2 font-mono text-[11px] text-[var(--text-muted)]">
                      等待编码...
                    </code>
                  </div>
                )}
              </div>
            </motion.div>

            {/* Decode Memory */}
            <motion.div variants={cardVariants} className="glass-panel p-6">
              <div className="mb-4 flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'var(--dim-x)26' }}
                >
                  <Eye className="h-5 w-5" style={{ color: 'var(--dim-x)' }} />
                </div>
                <div>
                  <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                    解码记忆
                  </h2>
                  <p className="font-body text-[12px] text-[var(--text-muted)]">
                    从 7D 晶格检索存储的向量
                  </p>
                </div>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                    记忆 ID
                  </label>
                  <Select
                    value={selectedId}
                    onValueChange={(v) => {
                      setSelectedId(v)
                      setDecodeResult(null)
                      setDecodeError('')
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="选择记忆..." />
                    </SelectTrigger>
                    <SelectContent>
                      {memories.map((m) => (
                        <SelectItem key={m.id} value={m.id}>
                          {m.raw}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {selected && (
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-3">
                    <div className="flex items-center gap-3 text-[11px]">
                      <span className="font-body text-[var(--text-muted)]">
                        锚点:
                      </span>
                      <span className="font-mono text-[var(--accent-cyan)]">
                        ({selected.coord.join(', ')})
                      </span>
                      <span className="font-body text-[var(--text-muted)]">
                        维度:
                      </span>
                      <Badge variant="outline" className="font-mono text-[10px]">
                        {selected.dims}D
                      </Badge>
                    </div>
                  </div>
                )}

                {decodeError && (
                  <div className="flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                    <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
                    <span className="font-body text-[12px] text-[var(--accent-red)]">
                      {decodeError}
                    </span>
                  </div>
                )}

                {decodeResult && (
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                    <h3 className="mb-3 font-display text-sm font-semibold text-[var(--text-primary)]">
                      解码结果
                    </h3>
                    <div className="space-y-2">
                      <div>
                        <span className="font-body text-[10px] text-[var(--text-muted)]">
                          解码向量
                        </span>
                        <p className="font-mono text-[11px] text-[var(--accent-cyan)]">
                          [{decodeResult.data.map((v) => v.toFixed(4)).join(', ')}]
                        </p>
                      </div>
                      <div className="flex items-center gap-2 pt-1">
                        <CheckCircle className="h-3.5 w-3.5 text-[var(--accent-green)]" />
                        <span className="font-body text-[10px] text-[var(--accent-green)]">
                          解码成功
                        </span>
                        <Badge variant="secondary" className="font-mono text-[10px]">
                          {decodeResult.data.length}D
                        </Badge>
                      </div>
                    </div>
                  </div>
                )}

                <Button
                  variant="outline"
                  className="w-full"
                  disabled={!selectedId || decodeLoading}
                  onClick={handleDecode}
                >
                  <Eye className="mr-2 h-4 w-4" />
                  {decodeLoading ? '解码中...' : '立即解码'}
                </Button>
              </div>
            </motion.div>
          </div>

          {/* ─── MEMORY LIBRARY ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                记忆库
              </h2>
              <Badge variant="outline" className="font-mono text-xs">
                {memories.length} 条记忆
              </Badge>
            </div>

            {listError && (
              <div className="mb-4 flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
                <span className="font-body text-[12px] text-[var(--accent-red)]">
                  {listError}
                </span>
              </div>
            )}

            <div className="max-h-[400px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>记忆 ID</TableHead>
                    <TableHead>锚点坐标</TableHead>
                    <TableHead>维度数</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {memories.map((mem) => (
                    <TableRow key={mem.id}>
                      <TableCell className="max-w-[200px] truncate font-mono text-xs">
                        {mem.id}
                      </TableCell>
                      <TableCell className="font-mono text-xs">
                        ({mem.coord.join(', ')})
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline" className="font-mono text-[10px]">
                          {mem.dims}D
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex items-center justify-end gap-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 px-2 text-xs"
                            onClick={() => setShowDetail(mem)}
                          >
                            <Eye className="mr-1 h-3 w-3" />
                            详情
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 px-2 text-xs"
                            onClick={() => handleCopy(mem.id)}
                          >
                            <Copy className="h-3 w-3" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 px-2 text-xs text-[var(--accent-red)]"
                            onClick={() => handleDelete(mem)}
                          >
                            <Trash2 className="h-3 w-3" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </motion.div>

          {/* ─── MEMORY DETAIL MODAL ─── */}
          {showDetail && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
              onClick={() => setShowDetail(null)}
            >
              <motion.div
                initial={{ scale: 0.9 }}
                animate={{ scale: 1 }}
                className="glass-panel w-full max-w-lg p-6"
                onClick={(e) => e.stopPropagation()}
              >
                <div className="mb-4 flex items-center justify-between">
                  <h3 className="font-display text-lg font-semibold text-[var(--text-primary)]">
                    记忆详情
                  </h3>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 w-7 p-0"
                    onClick={() => setShowDetail(null)}
                  >
                    <XCircle className="h-4 w-4" />
                  </Button>
                </div>

                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span className="font-body text-[12px] text-[var(--text-muted)]">
                      记忆 ID
                    </span>
                    <span className="max-w-[300px] truncate font-mono text-[12px] text-[var(--text-primary)]">
                      {showDetail.id}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="font-body text-[12px] text-[var(--text-muted)]">
                      锚点坐标
                    </span>
                    <span className="font-mono text-[12px] text-[var(--accent-cyan)]">
                      ({showDetail.coord.join(', ')})
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="font-body text-[12px] text-[var(--text-muted)]">
                      维度数
                    </span>
                    <span className="font-mono text-[12px]">{showDetail.dims}D</span>
                  </div>
                </div>

                <div className="mt-4 flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1"
                    onClick={() => handleCopy(showDetail.id)}
                  >
                    <Copy className="mr-1.5 h-3.5 w-3.5" />
                    复制 ID
                  </Button>
                  <Button
                    size="sm"
                    className="flex-1"
                    style={{ backgroundColor: 'var(--dim-x)' }}
                    onClick={async () => {
                      setSelectedId(showDetail.id)
                      setShowDetail(null)
                      if (!showDetail.dims || showDetail.dims === 0) {
                        setDecodeError('无法获取维度数')
                        return
                      }
                      setDecodeLoading(true)
                      setDecodeError('')
                      setDecodeResult(null)
                      try {
                        const anchor = showDetail.coord as [number, number, number]
                        const result = await api.decodeMemory(anchor, showDetail.dims)
                        if (result.data) {
                          setDecodeResult(result.data)
                        } else {
                          setDecodeError('解码失败：未找到对应记忆数据')
                        }
                      } catch (err: any) {
                        setDecodeError(err.message || '解码失败')
                      } finally {
                        setDecodeLoading(false)
                      }
                    }}
                  >
                    <Eye className="mr-1.5 h-3.5 w-3.5" />
                    {decodeLoading ? '解码中...' : '立即解码'}
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => handleDelete(showDetail)}
                  >
                    <Trash2 className="mr-1.5 h-3.5 w-3.5" />
                    删除
                  </Button>
                </div>
                <div className="mt-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="w-full"
                    onClick={() => setShowDetail(null)}
                  >
                    关闭
                  </Button>
                </div>
              </motion.div>
            </motion.div>
          )}
        </motion.div>
      </div>
    </div>
  )
}
