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
  Tag,
  Star,
  Clock,
  Link2,
  Edit3,
  Weight,
  Search,
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
import type {
  EncodeResult,
  DecodeResult,
  MemoryListItem,
  HebbianNeighborsResult,
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

interface MemoryItem {
  id: string
  coord: number[]
  dims: number
  tags: string[]
  category: string | null
  description: string | null
  importance: number
  created_at: number
}

function memoryItemFromApi(m: MemoryListItem): MemoryItem {
  const coordMatch = m.anchor.match(/\(\s*(-?[\d.]+)\s*,\s*(-?[\d.]+)\s*,\s*(-?[\d.]+)/)
  return {
    id: m.anchor,
    coord: coordMatch
      ? [Math.round(+coordMatch[1]), Math.round(+coordMatch[2]), Math.round(+coordMatch[3])]
      : [0, 0, 0],
    dims: m.data_dim,
    tags: m.tags || [],
    category: m.category,
    description: m.description,
    importance: m.importance,
    created_at: m.created_at,
  }
}

function formatTimestamp(ts: number): string {
  if (!ts) return '-'
  try {
    return new Date(ts).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return '-'
  }
}

function importanceColor(v: number): string {
  if (v >= 0.8) return 'var(--accent-green)'
  if (v >= 0.5) return 'var(--accent-cyan)'
  if (v >= 0.2) return 'var(--dim-mu)'
  return 'var(--text-muted)'
}

export default function Memory() {
  const [memories, setMemories] = useState<MemoryItem[]>([])
  const [selectedId, setSelectedId] = useState<string>('')
  const [encodeCoord, setEncodeCoord] = useState('100,100,100')
  const [encodeVector, setEncodeVector] = useState('[1.0, -2.5, 3.14]')
  const [precision, setPrecision] = useState([14])
  const [showDetail, setShowDetail] = useState<MemoryItem | null>(null)
  const [copied, setCopied] = useState(false)
  const [searchFilter, setSearchFilter] = useState('')

  const [encodeLoading, setEncodeLoading] = useState(false)
  const [encodeResult, setEncodeResult] = useState<EncodeResult['data'] | null>(null)
  const [encodeError, setEncodeError] = useState('')

  const [decodeLoading, setDecodeLoading] = useState(false)
  const [decodeResult, setDecodeResult] = useState<DecodeResult['data'] | null>(null)
  const [decodeError, setDecodeError] = useState('')

  const [listError, setListError] = useState('')

  const [hebbianNeighbors, setHebbianNeighbors] = useState<HebbianNeighborsResult['data'] | null>(null)
  const [hebbianLoading, setHebbianLoading] = useState(false)

  const [semanticResults, setSemanticResults] = useState<any[] | null>(null)
  const [semanticLoading, setSemanticLoading] = useState(false)

  const [editTags, setEditTags] = useState('')
  const [editCategory, setEditCategory] = useState('')
  const [editDescription, setEditDescription] = useState('')
  const [editImportance, setEditImportance] = useState('')
  const [saveLoading, setSaveLoading] = useState(false)
  const [saveMsg, setSaveMsg] = useState('')

  const [weightTarget, setWeightTarget] = useState('')
  const [weightBoost, setWeightBoost] = useState('0.5')
  const [weightLoading, setWeightLoading] = useState(false)
  const [weightMsg, setWeightMsg] = useState('')

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
    [memories, selectedId],
  )

  const filteredMemories = useMemo(() => {
    if (!searchFilter.trim()) return memories
    const q = searchFilter.toLowerCase()
    return memories.filter(
      (m) =>
        m.id.toLowerCase().includes(q) ||
        m.tags.some((t) => t.toLowerCase().includes(q)) ||
        (m.description && m.description.toLowerCase().includes(q)) ||
        (m.category && m.category.toLowerCase().includes(q)),
    )
  }, [memories, searchFilter])

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
        vector as number[],
      )
      setEncodeResult(result.data)
      loadMemories()
    } catch (err: any) {
      setEncodeError(err.message || '编码失败')
    } finally {
      setEncodeLoading(false)
    }
  }

  const handleDecode = async (item?: MemoryItem) => {
    const target = item || selected
    if (!target) return
    if (!target.dims || target.dims === 0) {
      setDecodeError('无法获取维度数，请选择一条有效记忆')
      return
    }
    setDecodeLoading(true)
    setDecodeError('')
    setDecodeResult(null)
    try {
      const anchor = target.coord as [number, number, number]
      const result = await api.decodeMemory(anchor, target.dims)
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

  const loadHebbianNeighbors = async (coord: number[]) => {
    if (coord.length < 3) return
    setHebbianLoading(true)
    setHebbianNeighbors(null)
    try {
      const res = await api.getHebbianNeighbors(coord[0], coord[1], coord[2])
      setHebbianNeighbors(res.data)
    } catch {
      setHebbianNeighbors(null)
    } finally {
      setHebbianLoading(false)
    }
  }

  const loadSemanticRelations = async (coord: number[]) => {
    if (coord.length < 3) return
    setSemanticLoading(true)
    setSemanticResults(null)
    try {
      const res = await api.semanticRelations(coord as [number, number, number])
      setSemanticResults(res.data?.results || [])
    } catch {
      setSemanticResults(null)
    } finally {
      setSemanticLoading(false)
    }
  }

  const openDetail = (item: MemoryItem) => {
    setShowDetail(item)
    setDecodeResult(null)
    setDecodeError('')
    setHebbianNeighbors(null)
    setSemanticResults(null)
    setEditTags(item.tags.join(', '))
    setEditCategory(item.category || '')
    setEditDescription(item.description || '')
    setEditImportance(String(item.importance))
    setSaveMsg('')
    setWeightTarget('')
    setWeightMsg('')
    loadHebbianNeighbors(item.coord)
    loadSemanticRelations(item.coord)
  }

  const handleSaveAnnotation = async () => {
    if (!showDetail) return
    setSaveLoading(true)
    setSaveMsg('')
    try {
      const tagsArr = editTags
        .split(',')
        .map((t) => t.trim())
        .filter(Boolean)
      const imp = parseFloat(editImportance)
      await api.annotateMemory(
        showDetail.coord as [number, number, number],
        tagsArr,
        editCategory || undefined,
        editDescription || undefined,
        isNaN(imp) ? undefined : imp,
      )
      setSaveMsg('保存成功')
      loadMemories()
    } catch (err: any) {
      setSaveMsg(err.message || '保存失败')
    } finally {
      setSaveLoading(false)
    }
  }

  const handleAdjustWeight = async () => {
    if (!showDetail || !weightTarget.trim()) return
    setWeightLoading(true)
    setWeightMsg('')
    try {
      const targetParts = weightTarget.split(',').map(Number)
      if (targetParts.length !== 3 || targetParts.some(isNaN)) {
        throw new Error('目标坐标格式错误')
      }
      const boost = parseFloat(weightBoost)
      if (isNaN(boost) || boost === 0) {
        throw new Error('boost 必须是非零数字')
      }
      const res = await api.adjustWeight(showDetail.coord, targetParts, boost)
      setWeightMsg(
        `权重: ${res.data.old_weight.toFixed(3)} → ${res.data.new_weight.toFixed(3)}`,
      )
      loadHebbianNeighbors(showDetail.coord)
    } catch (err: any) {
      setWeightMsg(err.message || '调整失败')
    } finally {
      setWeightLoading(false)
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
                    <p className="font-body text-[10px] text-[var(--text-muted)]">维度数</p>
                    <p className="font-mono text-lg font-bold text-[var(--text-primary)]">3</p>
                  </div>
                  <div className="rounded-lg bg-[var(--bg-surface)] p-3 text-center">
                    <p className="font-body text-[10px] text-[var(--text-muted)]">精度</p>
                    <p className="font-mono text-lg font-bold text-[var(--dim-mu)]">
                      1e-{precision[0]}
                    </p>
                  </div>
                  <div className="rounded-lg bg-[var(--bg-surface)] p-3 text-center">
                    <p className="font-body text-[10px] text-[var(--text-muted)]">能量消耗</p>
                    <p className="font-mono text-lg font-bold text-[var(--dim-e)]">2.4</p>
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

              <div className="mt-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                <h3 className="mb-2 font-display text-sm font-semibold text-[var(--text-primary)]">
                  编码结果
                </h3>
                {encodeResult ? (
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <span className="font-body text-[10px] text-[var(--text-muted)]">锚点</span>
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
                          {m.description || m.id}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {selected && (
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-3">
                    <div className="flex flex-wrap items-center gap-3 text-[11px]">
                      <span className="font-body text-[var(--text-muted)]">锚点:</span>
                      <span className="font-mono text-[var(--accent-cyan)]">
                        ({selected.coord.join(', ')})
                      </span>
                      <span className="font-body text-[var(--text-muted)]">维度:</span>
                      <Badge variant="outline" className="font-mono text-[10px]">
                        {selected.dims}D
                      </Badge>
                      <span className="font-body text-[var(--text-muted)]">重要性:</span>
                      <span
                        className="font-mono font-bold"
                        style={{ color: importanceColor(selected.importance) }}
                      >
                        {selected.importance.toFixed(2)}
                      </span>
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
                        <p className="max-h-[120px] overflow-auto font-mono text-[11px] text-[var(--accent-cyan)]">
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
                  onClick={() => handleDecode()}
                >
                  <Eye className="mr-2 h-4 w-4" />
                  {decodeLoading ? '解码中...' : '立即解码'}
                </Button>
              </div>
            </motion.div>
          </div>

          {/* MEMORY LIBRARY */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center justify-between gap-4">
              <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                记忆库
              </h2>
              <div className="flex items-center gap-3">
                <div className="relative">
                  <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-muted)]" />
                  <Input
                    value={searchFilter}
                    onChange={(e) => setSearchFilter(e.target.value)}
                    placeholder="搜索标签/描述/分类..."
                    className="h-8 w-[200px] pl-8 font-body text-xs"
                  />
                </div>
                <Badge variant="outline" className="font-mono text-xs">
                  {filteredMemories.length}/{memories.length} 条
                </Badge>
              </div>
            </div>

            {listError && (
              <div className="mb-4 flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
                <span className="font-body text-[12px] text-[var(--accent-red)]">
                  {listError}
                </span>
              </div>
            )}

            <div className="max-h-[500px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-[180px]">记忆 ID</TableHead>
                    <TableHead>锚点</TableHead>
                    <TableHead>描述</TableHead>
                    <TableHead>标签</TableHead>
                    <TableHead>分类</TableHead>
                    <TableHead className="text-center">重要性</TableHead>
                    <TableHead className="text-center">维度</TableHead>
                    <TableHead>创建时间</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredMemories.map((mem) => (
                    <TableRow
                      key={mem.id}
                      className="cursor-pointer hover:bg-[var(--bg-surface)]/50"
                      onClick={() => openDetail(mem)}
                    >
                      <TableCell className="max-w-[180px] truncate font-mono text-[11px]">
                        {mem.id}
                      </TableCell>
                      <TableCell className="font-mono text-[11px] text-[var(--accent-cyan)]">
                        ({mem.coord.join(', ')})
                      </TableCell>
                      <TableCell className="max-w-[150px] truncate text-[11px]">
                        {mem.description || '-'}
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-wrap gap-1">
                          {mem.tags.slice(0, 3).map((t) => (
                            <Badge
                              key={t}
                              variant="outline"
                              className="font-body text-[9px] px-1.5 py-0"
                            >
                              {t}
                            </Badge>
                          ))}
                          {mem.tags.length > 3 && (
                            <span className="font-body text-[9px] text-[var(--text-muted)]">
                              +{mem.tags.length - 3}
                            </span>
                          )}
                        </div>
                      </TableCell>
                      <TableCell className="text-[11px]">{mem.category || '-'}</TableCell>
                      <TableCell className="text-center">
                        <span
                          className="font-mono text-[11px] font-bold"
                          style={{ color: importanceColor(mem.importance) }}
                        >
                          {mem.importance.toFixed(2)}
                        </span>
                      </TableCell>
                      <TableCell className="text-center">
                        <Badge variant="outline" className="font-mono text-[10px]">
                          {mem.dims}D
                        </Badge>
                      </TableCell>
                      <TableCell className="font-body text-[10px] text-[var(--text-muted)]">
                        {formatTimestamp(mem.created_at)}
                      </TableCell>
                      <TableCell className="text-right" onClick={(e) => e.stopPropagation()}>
                        <div className="flex items-center justify-end gap-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 px-2 text-xs"
                            onClick={() => openDetail(mem)}
                          >
                            <Eye className="mr-1 h-3 w-3" />
                            详情
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

          {/* MEMORY DETAIL MODAL */}
          {showDetail && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="fixed inset-0 z-50 flex items-start justify-center overflow-auto bg-black/60 p-4 pt-8"
              onClick={() => setShowDetail(null)}
            >
              <motion.div
                initial={{ scale: 0.9, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                className="glass-panel w-full max-w-3xl p-6"
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

                {/* Basic Info */}
                <div className="mb-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  <h4 className="mb-3 flex items-center gap-2 font-display text-sm font-semibold text-[var(--text-primary)]">
                    <Database className="h-4 w-4" style={{ color: 'var(--dim-z)' }} />
                    基本信息
                  </h4>
                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        记忆 ID
                      </span>
                      <p className="truncate font-mono text-[11px] text-[var(--text-primary)]">
                        {showDetail.id}
                      </p>
                    </div>
                    <div>
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        锚点坐标
                      </span>
                      <p className="font-mono text-[11px] text-[var(--accent-cyan)]">
                        ({showDetail.coord.join(', ')})
                      </p>
                    </div>
                    <div>
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        数据维度
                      </span>
                      <p className="font-mono text-[11px]">{showDetail.dims}D</p>
                    </div>
                    <div>
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        重要性
                      </span>
                      <p
                        className="font-mono text-[11px] font-bold"
                        style={{ color: importanceColor(showDetail.importance) }}
                      >
                        {showDetail.importance.toFixed(3)}
                      </p>
                    </div>
                    <div>
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        分类
                      </span>
                      <p className="font-body text-[11px]">
                        {showDetail.category || (
                          <span className="text-[var(--text-muted)]">未分类</span>
                        )}
                      </p>
                    </div>
                    <div>
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        创建时间
                      </span>
                      <p className="flex items-center gap-1 font-body text-[11px]">
                        <Clock className="h-3 w-3 text-[var(--text-muted)]" />
                        {formatTimestamp(showDetail.created_at)}
                      </p>
                    </div>
                  </div>
                  {showDetail.description && (
                    <div className="mt-3">
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        描述
                      </span>
                      <p className="font-body text-[12px] text-[var(--text-primary)]">
                        {showDetail.description}
                      </p>
                    </div>
                  )}
                  {showDetail.tags.length > 0 && (
                    <div className="mt-3">
                      <span className="font-body text-[10px] text-[var(--text-muted)]">标签</span>
                      <div className="mt-1 flex flex-wrap gap-1.5">
                        {showDetail.tags.map((t) => (
                          <Badge
                            key={t}
                            variant="outline"
                            className="font-body text-[10px]"
                            style={{ borderColor: 'var(--dim-mu)', color: 'var(--dim-mu)' }}
                          >
                            <Tag className="mr-1 h-2.5 w-2.5" />
                            {t}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                </div>

                {/* Decode Result in Detail */}
                <div className="mb-4">
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    disabled={decodeLoading}
                    onClick={() => handleDecode(showDetail)}
                  >
                    <Eye className="mr-2 h-3.5 w-3.5" />
                    {decodeLoading
                      ? '解码中...'
                      : decodeResult
                        ? '重新解码'
                        : '解码数据向量'}
                  </Button>
                  {decodeError && (
                    <p className="mt-1 font-body text-[11px] text-[var(--accent-red)]">
                      {decodeError}
                    </p>
                  )}
                  {decodeResult && (
                    <div className="mt-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-3">
                      <span className="font-body text-[10px] text-[var(--text-muted)]">
                        数据向量 ({decodeResult.data.length}D)
                      </span>
                      <p className="mt-1 max-h-[100px] overflow-auto font-mono text-[11px] text-[var(--accent-cyan)]">
                        [{decodeResult.data.map((v) => v.toFixed(4)).join(', ')}]
                      </p>
                    </div>
                  )}
                </div>

                {/* Hebbian Neighbors */}
                <div className="mb-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  <h4 className="mb-3 flex items-center gap-2 font-display text-sm font-semibold text-[var(--text-primary)]">
                    <Link2 className="h-4 w-4" style={{ color: 'var(--dim-y)' }} />
                    Hebbian 关联
                  </h4>
                  {hebbianLoading ? (
                    <p className="font-body text-[11px] text-[var(--text-muted)]">加载中...</p>
                  ) : hebbianNeighbors && hebbianNeighbors.neighbors.length > 0 ? (
                    <div className="max-h-[200px] overflow-auto">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead>邻居坐标</TableHead>
                            <TableHead className="text-right">权重</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {hebbianNeighbors.neighbors.map((n, i) => (
                            <TableRow key={i}>
                              <TableCell className="font-mono text-[11px]">
                                {n.coord}
                              </TableCell>
                              <TableCell className="text-right font-mono text-[11px]">
                                <span
                                  style={{
                                    color:
                                      n.weight > 2
                                        ? 'var(--accent-green)'
                                        : n.weight > 1
                                          ? 'var(--accent-cyan)'
                                          : 'var(--text-muted)',
                                  }}
                                >
                                  {n.weight.toFixed(3)}
                                </span>
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </div>
                  ) : (
                    <p className="font-body text-[11px] text-[var(--text-muted)]">
                      无 Hebbian 关联
                    </p>
                  )}
                </div>

                {/* Semantic Relations */}
                <div className="mb-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  <h4 className="mb-3 flex items-center gap-2 font-display text-sm font-semibold text-[var(--text-primary)]">
                    <Search className="h-4 w-4" style={{ color: 'var(--dim-x)' }} />
                    语义关联
                  </h4>
                  {semanticLoading ? (
                    <p className="font-body text-[11px] text-[var(--text-muted)]">搜索中...</p>
                  ) : semanticResults && semanticResults.length > 0 ? (
                    <div className="max-h-[200px] overflow-auto">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead>锚点</TableHead>
                            <TableHead>描述</TableHead>
                            <TableHead className="text-right">相似度</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {semanticResults.map((r: any, i: number) => (
                            <TableRow key={i}>
                              <TableCell className="max-w-[120px] truncate font-mono text-[11px]">
                                {r.anchor}
                              </TableCell>
                              <TableCell className="max-w-[200px] truncate text-[11px]">
                                {r.description || '-'}
                              </TableCell>
                              <TableCell className="text-right font-mono text-[11px]">
                                <span
                                  style={{
                                    color:
                                      r.similarity > 0.8
                                        ? 'var(--accent-green)'
                                        : 'var(--accent-cyan)',
                                  }}
                                >
                                  {(r.similarity * 100).toFixed(1)}%
                                </span>
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </div>
                  ) : (
                    <p className="font-body text-[11px] text-[var(--text-muted)]">
                      无语义关联结果
                    </p>
                  )}
                </div>

                {/* Annotate / Edit */}
                <div className="mb-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  <h4 className="mb-3 flex items-center gap-2 font-display text-sm font-semibold text-[var(--text-primary)]">
                    <Edit3 className="h-4 w-4" style={{ color: 'var(--dim-w)' }} />
                    编辑标注
                  </h4>
                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <label className="mb-1 block font-body text-[10px] text-[var(--text-muted)]">
                        标签 (逗号分隔)
                      </label>
                      <Input
                        value={editTags}
                        onChange={(e) => setEditTags(e.target.value)}
                        className="font-body text-xs"
                        placeholder="tag1, tag2"
                      />
                    </div>
                    <div>
                      <label className="mb-1 block font-body text-[10px] text-[var(--text-muted)]">
                        分类
                      </label>
                      <Input
                        value={editCategory}
                        onChange={(e) => setEditCategory(e.target.value)}
                        className="font-body text-xs"
                        placeholder="分类名"
                      />
                    </div>
                    <div className="col-span-2">
                      <label className="mb-1 block font-body text-[10px] text-[var(--text-muted)]">
                        描述
                      </label>
                      <Input
                        value={editDescription}
                        onChange={(e) => setEditDescription(e.target.value)}
                        className="font-body text-xs"
                        placeholder="记忆描述"
                      />
                    </div>
                    <div>
                      <label className="mb-1 block font-body text-[10px] text-[var(--text-muted)]">
                        重要性 (0-1)
                      </label>
                      <Input
                        value={editImportance}
                        onChange={(e) => setEditImportance(e.target.value)}
                        className="font-mono text-xs"
                        type="number"
                        min="0"
                        max="1"
                        step="0.01"
                      />
                    </div>
                    <div className="flex items-end">
                      <Button
                        size="sm"
                        className="w-full"
                        style={{ backgroundColor: 'var(--dim-w)' }}
                        disabled={saveLoading}
                        onClick={handleSaveAnnotation}
                      >
                        <Star className="mr-1.5 h-3.5 w-3.5" />
                        {saveLoading ? '保存中...' : '保存标注'}
                      </Button>
                    </div>
                  </div>
                  {saveMsg && (
                    <p
                      className={`mt-2 font-body text-[11px] ${saveMsg === '保存成功' ? 'text-[var(--accent-green)]' : 'text-[var(--accent-red)]'}`}
                    >
                      {saveMsg}
                    </p>
                  )}
                </div>

                {/* Weight Adjustment */}
                <div className="mb-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                  <h4 className="mb-3 flex items-center gap-2 font-display text-sm font-semibold text-[var(--text-primary)]">
                    <Weight className="h-4 w-4" style={{ color: 'var(--dim-e)' }} />
                    Hebbian 权重调整
                  </h4>
                  <div className="grid grid-cols-3 gap-3">
                    <div>
                      <label className="mb-1 block font-body text-[10px] text-[var(--text-muted)]">
                        目标坐标 (x,y,z)
                      </label>
                      <Input
                        value={weightTarget}
                        onChange={(e) => setWeightTarget(e.target.value)}
                        className="font-mono text-xs"
                        placeholder="1,2,3"
                      />
                    </div>
                    <div>
                      <label className="mb-1 block font-body text-[10px] text-[var(--text-muted)]">
                        调整量 (-5~+5)
                      </label>
                      <Input
                        value={weightBoost}
                        onChange={(e) => setWeightBoost(e.target.value)}
                        className="font-mono text-xs"
                        type="number"
                        step="0.1"
                      />
                    </div>
                    <div className="flex items-end">
                      <Button
                        size="sm"
                        variant="outline"
                        className="w-full"
                        disabled={weightLoading || !weightTarget.trim()}
                        onClick={handleAdjustWeight}
                      >
                        {weightLoading ? '调整中...' : '调整'}
                      </Button>
                    </div>
                  </div>
                  {weightMsg && (
                    <p className="mt-2 font-mono text-[11px] text-[var(--accent-cyan)]">
                      {weightMsg}
                    </p>
                  )}
                </div>

                {/* Action Buttons */}
                <div className="flex gap-2">
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
                    variant="destructive"
                    size="sm"
                    className="flex-1"
                    onClick={() => handleDelete(showDetail)}
                  >
                    <Trash2 className="mr-1.5 h-3.5 w-3.5" />
                    删除记忆
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="flex-1"
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
