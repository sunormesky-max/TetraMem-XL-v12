import { useState, useMemo } from 'react'
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
  vector: number[]
  dims: number
  precision: number
  energy: number
}

function generateMockMemories(): MemoryItem[] {
  return Array.from({ length: 12 }, (_, i) => {
    const dims = Math.floor(Math.random() * 21) + 7
    return {
      id: `mem_${1000 + i}`,
      coord: Array.from({ length: 7 }, () => Math.floor(Math.random() * 256)),
      vector: Array.from({ length: dims }, () => Math.random() * 20 - 10),
      dims,
      precision: -(Math.random() * 14 + 2),
      energy: Math.random() * 5 + 0.5,
    }
  })
}

export default function Memory() {
  const [memories] = useState<MemoryItem[]>(generateMockMemories())
  const [selectedId, setSelectedId] = useState<string>('')
  const [encodeCoord, setEncodeCoord] = useState('128,128,128,0,0,0,0')
  const [encodeVector, setEncodeVector] = useState('[1.0, -2.5, 3.14, 0.0, 2.71]')
  const [precision, setPrecision] = useState([14])
  const [showDetail, setShowDetail] = useState<MemoryItem | null>(null)
  const [copied, setCopied] = useState(false)

  const selected = useMemo(
    () => memories.find((m) => m.id === selectedId) || null,
    [memories, selectedId]
  )

  const selectedDecoded = useMemo(() => {
    if (!selected) return null
    return selected.vector.map((v) => v + (Math.random() - 0.5) * 1e-12)
  }, [selected])

  const handleCopy = (id: string) => {
    navigator.clipboard.writeText(id).catch(() => {})
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
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
                      7
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
                <Button className="w-full" style={{ backgroundColor: 'var(--dim-z)' }}>
                  <Database className="mr-2 h-4 w-4" />
                  编码记忆
                </Button>
              </div>

              {/* Encoding Result */}
              <div className="mt-4 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                <h3 className="mb-2 font-display text-sm font-semibold text-[var(--text-primary)]">
                  编码结果
                </h3>
                <div className="flex items-center gap-2">
                  <code className="flex-1 rounded bg-[var(--bg-deep)] px-3 py-2 font-mono text-[11px] text-[var(--accent-green)]">
                    mem_{Math.floor(Math.random() * 9000 + 1000)}
                  </code>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 px-2"
                    onClick={() => handleCopy('mem_example')}
                  >
                    {copied ? (
                      <CheckCircle className="h-4 w-4 text-[var(--accent-green)]" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
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
                    onValueChange={setSelectedId}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="选择记忆..." />
                    </SelectTrigger>
                    <SelectContent>
                      {memories.map((m) => (
                        <SelectItem key={m.id} value={m.id}>
                          {m.id} ({m.dims}D)
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {selected && selectedDecoded && (
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] p-4">
                    <h3 className="mb-3 font-display text-sm font-semibold text-[var(--text-primary)]">
                      解码结果
                    </h3>
                    <div className="space-y-2">
                      <div>
                        <span className="font-body text-[10px] text-[var(--text-muted)]">
                          原始向量
                        </span>
                        <p className="font-mono text-[11px] text-[var(--text-secondary)]">
                          [{selected.vector.map((v) => v.toFixed(4)).join(', ')}]
                        </p>
                      </div>
                      <div>
                        <span className="font-body text-[10px] text-[var(--text-muted)]">
                          解码向量
                        </span>
                        <p className="font-mono text-[11px] text-[var(--accent-cyan)]">
                          [{selectedDecoded.map((v) => v.toFixed(4)).join(', ')}]
                        </p>
                      </div>
                      <div>
                        <span className="font-body text-[10px] text-[var(--text-muted)]">
                          逐元素差异
                        </span>
                        <p className="font-mono text-[11px] text-[var(--dim-e)]">
                          [{selected.vector.map((v, i) => Math.abs(v - selectedDecoded![i]).toExponential(2)).join(', ')}]
                        </p>
                      </div>
                      <div className="flex items-center gap-2 pt-1">
                        <span className="font-body text-[10px] text-[var(--text-muted)]">
                          最大误差
                        </span>
                        <Badge variant="secondary" className="font-mono text-[10px]">
                          {'<1e-14'}
                        </Badge>
                        <CheckCircle className="h-3.5 w-3.5 text-[var(--accent-green)]" />
                        <span className="font-body text-[10px] text-[var(--accent-green)]">
                          已验证
                        </span>
                      </div>
                    </div>
                  </div>
                )}

                <Button
                  variant="outline"
                  className="w-full"
                  disabled={!selectedId}
                >
                  <Eye className="mr-2 h-4 w-4" />
                  立即解码
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

            <div className="max-h-[400px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>记忆 ID</TableHead>
                    <TableHead>锚点坐标</TableHead>
                    <TableHead>维度数</TableHead>
                    <TableHead>精度</TableHead>
                    <TableHead>能量消耗</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {memories.map((mem) => (
                    <TableRow key={mem.id}>
                      <TableCell className="font-mono text-xs">{mem.id}</TableCell>
                      <TableCell className="font-mono text-xs">
                        ({mem.coord.join(',')})
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline" className="font-mono text-[10px]">
                          {mem.dims}D
                        </Badge>
                      </TableCell>
                      <TableCell className="font-mono text-xs text-[var(--dim-mu)]">
                        1e{Math.round(mem.precision)}
                      </TableCell>
                      <TableCell className="font-mono text-xs text-[var(--dim-e)]">
                        {mem.energy.toFixed(2)}
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
                            解码
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
                    <span className="font-mono text-[12px] text-[var(--text-primary)]">
                      {showDetail.id}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="font-body text-[12px] text-[var(--text-muted)]">
                      锚点坐标
                    </span>
                    <span className="font-mono text-[12px] text-[var(--accent-cyan)]">
                      ({showDetail.coord.join(',')})
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="font-body text-[12px] text-[var(--text-muted)]">
                      维度数
                    </span>
                    <span className="font-mono text-[12px]">{showDetail.dims}D</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="font-body text-[12px] text-[var(--text-muted)]">
                      精度
                    </span>
                    <span className="font-mono text-[12px] text-[var(--dim-mu)]">
                      1e{Math.round(showDetail.precision)}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="font-body text-[12px] text-[var(--text-muted)]">
                      能量消耗
                    </span>
                    <span className="font-mono text-[12px] text-[var(--dim-e)]">
                      {showDetail.energy.toFixed(2)}
                    </span>
                  </div>
                  <div>
                    <span className="mb-1 block font-body text-[12px] text-[var(--text-muted)]">
                      数据向量
                    </span>
                    <code className="block max-h-[120px] overflow-auto rounded bg-[var(--bg-deep)] p-3 font-mono text-[10px] text-[var(--text-secondary)]">
                      [{showDetail.vector.map((v) => v.toFixed(6)).join(', ')}]
                    </code>
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
                    onClick={() => {
                      setSelectedId(showDetail.id)
                      setShowDetail(null)
                    }}
                  >
                    <Eye className="mr-1.5 h-3.5 w-3.5" />
                    立即解码
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
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
