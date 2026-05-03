import { useState, useCallback, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Search,
  Brain,
  Link,
  Sparkles,
  Loader2,
  XCircle,
  CheckCircle,
  Target,
  Route,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Slider } from '@/components/ui/slider'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  api,
  type SemanticStatusResult,
  type SemanticSearchResult,
  type RecallResult,
  type AssociateResult,
} from '@/services/api'

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

export default function Semantic() {
  const [status, setStatus] = useState<SemanticStatusResult['data'] | null>(null)
  const [statusLoading, setStatusLoading] = useState(false)

  const [queryText, setQueryText] = useState('')
  const [searchLoading, setSearchLoading] = useState(false)
  const [searchResults, setSearchResults] = useState<SemanticSearchResult['data']['results'] | null>(null)
  const [searchError, setSearchError] = useState('')

  const [recallQuery, setRecallQuery] = useState('')
  const [recallLimit, setRecallLimit] = useState([5])
  const [recallLoading, setRecallLoading] = useState(false)
  const [recallResults, setRecallResults] = useState<RecallResult['data'] | null>(null)
  const [recallError, setRecallError] = useState('')

  const [assocTopic, setAssocTopic] = useState('')
  const [assocLoading, setAssocLoading] = useState(false)
  const [assocResults, setAssocResults] = useState<AssociateResult['data'] | null>(null)
  const [assocError, setAssocError] = useState('')

  const loadStatus = useCallback(async () => {
    setStatusLoading(true)
    try {
      const res = await api.semanticStatus()
      if (res.success) setStatus(res.data)
    } catch {
      // silent
    }
    setStatusLoading(false)
  }, [])

  useEffect(() => {
    loadStatus()
  }, [loadStatus])

  const handleSearch = useCallback(async () => {
    if (!queryText.trim()) return
    setSearchLoading(true)
    setSearchError('')
    setSearchResults(null)
    try {
      const res = await api.semanticQuery(queryText.trim())
      if (res.success) {
        setSearchResults(res.data.results)
      }
    } catch (err: any) {
      setSearchError(err.message || '语义搜索失败')
    }
    setSearchLoading(false)
  }, [queryText])

  const handleRecall = useCallback(async () => {
    if (!recallQuery.trim()) return
    setRecallLoading(true)
    setRecallError('')
    setRecallResults(null)
    try {
      const res = await api.recall(recallQuery.trim(), recallLimit[0])
      if (res.success) {
        setRecallResults(res.data)
      }
    } catch (err: any) {
      setRecallError(err.message || '回忆失败')
    }
    setRecallLoading(false)
  }, [recallQuery, recallLimit])

  const handleAssociate = useCallback(async () => {
    if (!assocTopic.trim()) return
    setAssocLoading(true)
    setAssocError('')
    setAssocResults(null)
    try {
      const res = await api.associate(assocTopic.trim())
      if (res.success) {
        setAssocResults(res.data)
      }
    } catch (err: any) {
      setAssocError(err.message || '关联发现失败')
    }
    setAssocLoading(false)
  }, [assocTopic])

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
          <motion.div variants={cardVariants} className="flex items-center gap-4">
            <div
              className="flex h-12 w-12 items-center justify-center rounded-full"
              style={{ backgroundColor: 'var(--accent-cyan)26' }}
            >
              <Brain className="h-6 w-6" style={{ color: 'var(--accent-cyan)' }} />
            </div>
            <div>
              <h1 className="font-display text-2xl font-bold text-[var(--text-primary)]">
                语义搜索
              </h1>
              <p className="font-body text-[12px] text-[var(--text-muted)]">
                Semantic Search & Knowledge
              </p>
            </div>
          </motion.div>

          {/* ─── STATUS CARDS ─── */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {[
              {
                label: 'Embeddings Indexed',
                value: status ? String(status.embeddings_indexed) : statusLoading ? '...' : '--',
                icon: Target,
                color: 'var(--accent-cyan)',
              },
              {
                label: 'Relations Total',
                value: status ? String(status.relations_total) : '--',
                icon: Link,
                color: 'var(--accent-green)',
              },
              {
                label: 'Concepts Extracted',
                value: status ? String(status.concepts_extracted) : '--',
                icon: Sparkles,
                color: 'var(--dim-e, var(--accent-cyan))',
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

          {/* ─── TEXT QUERY SEARCH ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-cyan)26' }}
              >
                <Search className="h-5 w-5" style={{ color: 'var(--accent-cyan)' }} />
              </div>
              <div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  文本搜索
                </h2>
                <p className="font-body text-[12px] text-[var(--text-muted)]">
                  输入自然语言文本进行语义搜索
                </p>
              </div>
            </div>

            <div className="flex items-end gap-3">
              <div className="flex-1">
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  搜索文本
                </label>
                <input
                  type="text"
                  value={queryText}
                  onChange={(e) => setQueryText(e.target.value)}
                  placeholder="输入搜索内容..."
                  className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none"
                  onKeyDown={(e) => { if (e.key === 'Enter') handleSearch() }}
                />
              </div>
              <Button
                disabled={searchLoading || !queryText.trim()}
                onClick={handleSearch}
                style={{ backgroundColor: 'var(--accent-cyan)' }}
              >
                {searchLoading ? (
                  <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                ) : (
                  <Search className="mr-1.5 h-4 w-4" />
                )}
                搜索
              </Button>
            </div>

            {searchError && (
              <div className="mt-4 flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
                <span className="font-body text-[12px] text-[var(--accent-red)]">{searchError}</span>
              </div>
            )}

            {searchResults !== null && (
              <div className="mt-4 max-h-[400px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Anchor</TableHead>
                      <TableHead>Similarity</TableHead>
                      <TableHead>Distance</TableHead>
                      <TableHead>Tags</TableHead>
                      <TableHead>Category</TableHead>
                      <TableHead>Description</TableHead>
                      <TableHead>Importance</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {searchResults.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={7} className="py-8 text-center">
                          <Search className="mx-auto mb-2 h-8 w-8 text-[var(--text-muted)] opacity-40" />
                          <p className="font-body text-[12px] text-[var(--text-muted)]">
                            未找到相关结果
                          </p>
                        </TableCell>
                      </TableRow>
                    ) : (
                      searchResults.map((r) => (
                        <TableRow key={r.anchor}>
                          <TableCell className="max-w-[180px] truncate font-mono text-xs text-[var(--accent-cyan)]">
                            {r.anchor}
                          </TableCell>
                          <TableCell className="font-mono text-xs">
                            <Badge
                              variant="outline"
                              className="font-mono text-[10px]"
                              style={{ color: 'var(--accent-green)', borderColor: 'var(--accent-green)' }}
                            >
                              {r.similarity.toFixed(4)}
                            </Badge>
                          </TableCell>
                          <TableCell className="font-mono text-xs text-[var(--text-secondary)]">
                            {r.distance.toFixed(4)}
                          </TableCell>
                          <TableCell>
                            <div className="flex flex-wrap gap-1">
                              {r.tags.map((t) => (
                                <Badge key={t} variant="secondary" className="font-body text-[10px]">
                                  {t}
                                </Badge>
                              ))}
                            </div>
                          </TableCell>
                          <TableCell className="font-body text-xs text-[var(--text-secondary)]">
                            {r.category ?? '--'}
                          </TableCell>
                          <TableCell className="max-w-[200px] truncate font-body text-xs text-[var(--text-secondary)]">
                            {r.description ?? '--'}
                          </TableCell>
                          <TableCell className="font-mono text-xs">
                            <Badge variant="outline" className="font-mono text-[10px]">
                              {r.importance.toFixed(2)}
                            </Badge>
                          </TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </div>
            )}
          </motion.div>

          {/* ─── RECALL SECTION ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--accent-green)26' }}
              >
                <Brain className="h-5 w-5" style={{ color: 'var(--accent-green)' }} />
              </div>
              <div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  记忆回忆
                </h2>
                <p className="font-body text-[12px] text-[var(--text-muted)]">
                  通过语义关联回忆相关记忆
                </p>
              </div>
            </div>

            <div className="space-y-4">
              <div className="flex items-end gap-3">
                <div className="flex-1">
                  <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                    查询内容
                  </label>
                  <input
                    type="text"
                    value={recallQuery}
                    onChange={(e) => setRecallQuery(e.target.value)}
                    placeholder="输入回忆查询..."
                    className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none"
                    onKeyDown={(e) => { if (e.key === 'Enter') handleRecall() }}
                  />
                </div>
                <Button
                  disabled={recallLoading || !recallQuery.trim()}
                  onClick={handleRecall}
                  style={{ backgroundColor: 'var(--accent-green)' }}
                >
                  {recallLoading ? (
                    <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                  ) : (
                    <Brain className="mr-1.5 h-4 w-4" />
                  )}
                  回忆
                </Button>
              </div>

              <div>
                <label className="mb-2 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  返回数量: {recallLimit[0]}
                </label>
                <Slider
                  value={recallLimit}
                  onValueChange={setRecallLimit}
                  min={1}
                  max={20}
                  step={1}
                />
              </div>
            </div>

            {recallError && (
              <div className="mt-4 flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
                <span className="font-body text-[12px] text-[var(--accent-red)]">{recallError}</span>
              </div>
            )}

            {recallResults && (
              <div className="mt-4">
                <div className="mb-3 flex items-center gap-2">
                  <CheckCircle className="h-4 w-4 text-[var(--accent-green)]" />
                  <span className="font-body text-[12px] text-[var(--accent-green)]">
                    查询 "{recallResults.query}" 返回 {recallResults.total} 条结果
                  </span>
                </div>
                <div className="max-h-[400px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Anchor</TableHead>
                        <TableHead>Method</TableHead>
                        <TableHead>Similarity</TableHead>
                        <TableHead>Description</TableHead>
                        <TableHead>Tags</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {recallResults.results.length === 0 ? (
                        <TableRow>
                          <TableCell colSpan={5} className="py-8 text-center">
                            <p className="font-body text-[12px] text-[var(--text-muted)]">
                              未找到相关记忆
                            </p>
                          </TableCell>
                        </TableRow>
                      ) : (
                        recallResults.results.map((r) => (
                          <TableRow key={r.anchor}>
                            <TableCell className="max-w-[180px] truncate font-mono text-xs text-[var(--accent-cyan)]">
                              {r.anchor}
                            </TableCell>
                            <TableCell>
                              <Badge
                                variant="outline"
                                className="font-mono text-[10px]"
                                style={{
                                  color: r.method === 'spatial' ? 'var(--accent-cyan)' : 'var(--accent-green)',
                                  borderColor: r.method === 'spatial' ? 'var(--accent-cyan)' : 'var(--accent-green)',
                                }}
                              >
                                {r.method === 'spatial' ? 'spatial' : 'knn'}
                              </Badge>
                            </TableCell>
                            <TableCell className="font-mono text-xs">
                              {r.similarity.toFixed(4)}
                            </TableCell>
                            <TableCell className="max-w-[200px] truncate font-body text-xs text-[var(--text-secondary)]">
                              {r.description || '--'}
                            </TableCell>
                            <TableCell>
                              <div className="flex flex-wrap gap-1">
                                {r.tags.map((t) => (
                                  <Badge key={t} variant="secondary" className="font-body text-[10px]">
                                    {t}
                                  </Badge>
                                ))}
                              </div>
                            </TableCell>
                          </TableRow>
                        ))
                      )}
                    </TableBody>
                  </Table>
                </div>
              </div>
            )}
          </motion.div>

          {/* ─── ASSOCIATE SECTION ─── */}
          <motion.div variants={cardVariants} className="glass-panel p-6">
            <div className="mb-4 flex items-center gap-3">
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full"
                style={{ backgroundColor: 'var(--dim-e, var(--accent-cyan))26' }}
              >
                <Link className="h-5 w-5" style={{ color: 'var(--dim-e, var(--accent-cyan))' }} />
              </div>
              <div>
                <h2 className="font-display text-xl font-semibold text-[var(--text-primary)]">
                  关联发现
                </h2>
                <p className="font-body text-[12px] text-[var(--text-muted)]">
                  探索主题相关的记忆关联网络
                </p>
              </div>
            </div>

            <div className="flex items-end gap-3">
              <div className="flex-1">
                <label className="mb-1 block font-body text-[12px] font-semibold text-[var(--text-secondary)]">
                  主题
                </label>
                <input
                  type="text"
                  value={assocTopic}
                  onChange={(e) => setAssocTopic(e.target.value)}
                  placeholder="输入关联主题..."
                  className="w-full rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-deep)] px-3 py-2 font-mono text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--accent-cyan)] focus:outline-none"
                  onKeyDown={(e) => { if (e.key === 'Enter') handleAssociate() }}
                />
              </div>
              <Button
                disabled={assocLoading || !assocTopic.trim()}
                onClick={handleAssociate}
                style={{ backgroundColor: 'var(--dim-e, var(--accent-cyan))' }}
              >
                {assocLoading ? (
                  <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                ) : (
                  <Link className="mr-1.5 h-4 w-4" />
                )}
                关联发现
              </Button>
            </div>

            {assocError && (
              <div className="mt-4 flex items-center gap-2 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 px-3 py-2">
                <XCircle className="h-4 w-4 text-[var(--accent-red)]" />
                <span className="font-body text-[12px] text-[var(--accent-red)]">{assocError}</span>
              </div>
            )}

            {assocResults && (
              <div className="mt-4 space-y-4">
                <div className="flex items-center gap-3">
                  <CheckCircle className="h-4 w-4 text-[var(--accent-green)]" />
                  <span className="font-body text-[12px] text-[var(--accent-green)]">
                    主题 "{assocResults.topic}" — 种子锚点:
                  </span>
                  <code className="rounded bg-[var(--bg-deep)] px-2 py-1 font-mono text-[11px] text-[var(--accent-cyan)]">
                    {assocResults.seed_anchor}
                  </code>
                  <Badge variant="outline" className="font-mono text-[10px]">
                    {assocResults.total} 条关联
                  </Badge>
                </div>

                <div className="max-h-[400px] overflow-auto rounded-lg border border-[var(--border-subtle)]">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Source</TableHead>
                        <TableHead>Confidence</TableHead>
                        <TableHead>Hops</TableHead>
                        <TableHead>Targets</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {assocResults.associations.length === 0 ? (
                        <TableRow>
                          <TableCell colSpan={4} className="py-8 text-center">
                            <p className="font-body text-[12px] text-[var(--text-muted)]">
                              未发现关联
                            </p>
                          </TableCell>
                        </TableRow>
                      ) : (
                        assocResults.associations.map((a, i) => (
                          <TableRow key={i}>
                            <TableCell className="max-w-[180px] truncate font-mono text-xs text-[var(--accent-cyan)]">
                              {a.source}
                            </TableCell>
                            <TableCell>
                              <Badge
                                variant="outline"
                                className="font-mono text-[10px]"
                                style={{
                                  color: a.confidence > 0.7 ? 'var(--accent-green)' : 'var(--text-secondary)',
                                  borderColor: a.confidence > 0.7 ? 'var(--accent-green)' : 'var(--border-subtle)',
                                }}
                              >
                                {a.confidence.toFixed(4)}
                              </Badge>
                            </TableCell>
                            <TableCell>
                              <div className="flex items-center gap-1">
                                <Route className="h-3 w-3 text-[var(--text-muted)]" />
                                <span className="font-mono text-xs">{a.hops}</span>
                              </div>
                            </TableCell>
                            <TableCell>
                              <div className="flex flex-wrap gap-1">
                                {a.targets.map((t) => (
                                  <Badge key={t.anchor} variant="secondary" className="font-body text-[10px]">
                                    {t.description || t.anchor}
                                  </Badge>
                                ))}
                              </div>
                            </TableCell>
                          </TableRow>
                        ))
                      )}
                    </TableBody>
                  </Table>
                </div>
              </div>
            )}
          </motion.div>
        </motion.div>
      </div>
    </div>
  )
}
