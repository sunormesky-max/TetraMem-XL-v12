import { useEffect, useState } from 'react'
import { useLocation } from 'react-router-dom'


const routeNames: Record<string, string> = {
  '/': '仪表盘',
  '/universe': '宇宙',
  '/memory': '记忆',
  '/pulse': '脉冲',
  '/dream': '梦境',
  '/topology': '拓扑',
  '/regulation': '调节',
  '/cluster': '集群',
  '/timeline': '时间轴',
  '/api': 'API 演练场',
}

const routeBreadcrumbs: Record<string, string> = {
  '/': '系统概览',
  '/universe': '7D 可视化',
  '/memory': '编码 / 解码',
  '/pulse': 'PCNN 引擎',
  '/dream': '梦境引擎',
  '/topology': '拓扑分析',
  '/regulation': '能量调节',
  '/cluster': 'Raft 集群管理',
  '/timeline': '记忆时间轴 + 溯源',
  '/api': '端点浏览器',
}

export default function Header() {
  const location = useLocation()
  const [time, setTime] = useState(new Date())

  useEffect(() => {
    const interval = setInterval(() => setTime(new Date()), 1000)
    return () => clearInterval(interval)
  }, [])

  const pageName = routeNames[location.pathname] || 'Dashboard'
  const breadcrumb = routeBreadcrumbs[location.pathname] || ''

  const hours = String(time.getHours()).padStart(2, '0')
  const minutes = String(time.getMinutes()).padStart(2, '0')
  const seconds = String(time.getSeconds()).padStart(2, '0')

  return (
    <header
      className="sticky top-0 z-30 flex h-header items-center justify-between border-b border-[var(--border-subtle)] bg-[var(--bg-deep)] px-6"
      style={{ backgroundImage: 'var(--gradient-hero)' }}
    >
      {/* Left: Title + Breadcrumb */}
      <div>
        <h1 className="font-display text-xl font-semibold leading-tight tracking-[-0.01em] text-[var(--text-primary)] md:text-2xl">
          {pageName}
        </h1>
        <p className="mt-0.5 font-body text-xs text-[var(--text-muted)]">
          {breadcrumb}
        </p>
      </div>

      {/* Right: Energy Badge + Status + Clock */}
      <div className="flex items-center gap-4">
        {/* Energy Conservation Badge */}
        <div
          className="hidden items-center gap-2 rounded-full px-4 py-1.5 sm:flex"
          style={{
            background: 'linear-gradient(90deg, #00E5FF 0%, #FFD600 50%, #F50057 100%)',
            boxShadow: '0 0 12px rgba(0, 229, 255, 0.3)',
          }}
        >
          <span className="whitespace-nowrap font-body text-[11px] font-semibold uppercase tracking-[0.06em] text-white">
            能量守恒 &#10003;
          </span>
        </div>

        {/* System Status */}
        <div className="flex items-center gap-2 rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 py-1.5">
          <span className="relative flex h-2.5 w-2.5">
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--accent-green)] opacity-75" />
            <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-[var(--accent-green)]" />
          </span>
          <span className="hidden font-body text-xs text-[var(--text-secondary)] sm:inline">
            在线
          </span>
        </div>

        {/* Clock */}
        <div className="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 py-1.5">
          <span className="font-mono text-lg font-semibold leading-none tracking-[-0.01em] text-[var(--text-primary)]">
            {hours}
            <span className="animate-colon-blink text-[var(--accent-cyan)]">:</span>
            {minutes}
            <span className="animate-colon-blink text-[var(--accent-cyan)]">:</span>
            {seconds}
          </span>
        </div>


      </div>
    </header>
  )
}
