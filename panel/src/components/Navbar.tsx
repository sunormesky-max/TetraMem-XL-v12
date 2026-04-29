import { NavLink } from 'react-router-dom'
import {
  LayoutDashboard,
  Globe,
  Database,
  Zap,
  Moon,
  Network,
  Activity,
  Code2,
  Box,
  Server,
  Clock,
} from 'lucide-react'

const navSections = [
  {
    label: '概览',
    items: [
      { to: '/', icon: LayoutDashboard, label: '仪表盘' },
      { to: '/universe', icon: Globe, label: '宇宙' },
      { to: '/memory', icon: Database, label: '记忆' },
    ],
  },
  {
    label: '系统',
    items: [
      { to: '/pulse', icon: Zap, label: '脉冲' },
      { to: '/dream', icon: Moon, label: '梦境' },
      { to: '/topology', icon: Network, label: '拓扑' },
      { to: '/regulation', icon: Activity, label: '调节' },
      { to: '/cluster', icon: Server, label: '集群' },
      { to: '/timeline', icon: Clock, label: '时间轴' },
    ],
  },
  {
    label: '开发者',
    items: [
      { to: '/api', icon: Code2, label: 'API 演练场' },
    ],
  },
]

export default function Navbar() {
  return (
    <aside
      className="fixed left-0 top-0 z-40 flex h-screen w-[var(--sidebar-width)] flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-surface)]/[0.8] backdrop-blur-xl"
      style={{ backdropFilter: 'blur(20px)' }}
    >
      {/* Logo */}
      <div
        className="flex h-header shrink-0 items-center gap-3 border-b border-[var(--border-subtle)] px-5"
      >
        <Box className="h-6 w-6 text-[var(--accent-cyan)]" style={{ filter: 'drop-shadow(0 0 6px rgba(0,229,255,0.5))' }} />
        <div className="flex flex-col">
          <span
            className="font-display text-base font-semibold tracking-wider text-[var(--text-primary)]"
          >
            TetraMem
          </span>
          <span className="font-mono text-[9px] tracking-widest text-[var(--text-muted)]">
            XL v12.0
          </span>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto py-4">
        {navSections.map((section) => (
          <div key={section.label} className="mb-6">
            <span
              className="mb-2 block px-5 font-body text-[11px] font-semibold uppercase tracking-[0.06em] text-[var(--text-muted)]"
            >
              {section.label}
            </span>
            <ul className="space-y-0.5 px-3">
              {section.items.map((item) => (
                <li key={item.to}>
                  <NavLink
                    to={item.to}
                    end={item.to === '/'}
                    className={({ isActive }) =>
                      [
                        'group flex h-11 items-center gap-3 rounded-lg px-4 font-display text-sm font-medium transition-all duration-200 ease-out',
                        isActive
                          ? 'border-l-[3px] border-l-[var(--accent-cyan)] bg-[rgba(0,229,255,0.1)] text-[var(--accent-cyan)]'
                          : 'border-l-[3px] border-l-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-surface-hover)] hover:text-[var(--text-primary)] hover:translate-x-1',
                      ].join(' ')
                    }
                  >
                    <item.icon className="h-5 w-5 shrink-0" />
                    <span>{item.label}</span>
                  </NavLink>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>

      {/* Footer */}
      <div className="shrink-0 border-t border-[var(--border-subtle)] px-5 py-4">
        <div className="flex items-center gap-2">
          <span className="relative flex h-2 w-2">
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--accent-green)] opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-[var(--accent-green)]" />
          </span>
          <span className="font-body text-[11px] text-[var(--text-muted)]">
            系统在线
          </span>
        </div>
        <p className="mt-1 font-mono text-[10px] text-[var(--text-muted)]">
          7D 宇宙已激活
        </p>
      </div>
    </aside>
  )
}
