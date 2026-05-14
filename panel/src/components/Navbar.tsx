import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Database,
  Globe,
  Zap,
  Moon,
  Brain,
  Heart,
  Eye,
  Atom,
  Network,
  Activity,
  Server,
  Clock,
  Shield,
  Puzzle,
  Terminal,
} from "lucide-react";

interface NavItem {
  path: string;
  label: string;
  icon: typeof Zap;
}

const navGroups: { title: string; items: NavItem[] }[] = [
  {
    title: "核心",
    items: [
      { path: "/", label: "仪表盘", icon: LayoutDashboard },
      { path: "/memory", label: "记忆", icon: Database },
      { path: "/universe", label: "宇宙", icon: Globe },
    ],
  },
  {
    title: "认知",
    items: [
      { path: "/pulse", label: "脉冲", icon: Zap },
      { path: "/dream", label: "梦境", icon: Moon },
      { path: "/semantic", label: "语义", icon: Brain },
      { path: "/emotion", label: "情感", icon: Heart },
    ],
  },
  {
    title: "暗宇宙",
    items: [
      { path: "/dark", label: "暗物质", icon: Eye },
      { path: "/physics", label: "物理", icon: Atom },
    ],
  },
  {
    title: "系统",
    items: [
      { path: "/topology", label: "拓扑", icon: Network },
      { path: "/regulation", label: "调节", icon: Activity },
      { path: "/cluster", label: "集群", icon: Server },
      { path: "/timeline", label: "时间轴", icon: Clock },
    ],
  },
  {
    title: "管理",
    items: [
      { path: "/watchdog", label: "看门狗", icon: Shield },
      { path: "/plugins", label: "插件", icon: Puzzle },
      { path: "/api", label: "API", icon: Terminal },
    ],
  },
];

export default function Navbar() {
  return (
    <nav className="sticky top-0 flex h-screen w-[var(--sidebar-width)] shrink-0 flex-col border-r border-[var(--border-subtle)] bg-[var(--bg-deep)]">
      <div className="flex items-center gap-3 px-5 py-6">
        <div className="flex h-9 w-9 items-center justify-center rounded-lg energy-badge-glow">
          <Zap className="h-5 w-5 text-white" />
        </div>
        <div>
          <h1 className="font-display text-sm font-bold text-[var(--text-primary)]">
            TetraMem-XL
          </h1>
          <p className="font-mono text-[10px] text-[var(--text-muted)]">v12.0</p>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-3 pb-4">
        {navGroups.map((group) => (
          <div key={group.title} className="mb-4">
            <p className="mb-1 px-2 font-body text-[10px] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
              {group.title}
            </p>
            {group.items.map((item) => (
              <NavLink
                key={item.path}
                to={item.path}
                end={item.path === "/"}
                className={({ isActive }) =>
                  [
                    "flex items-center gap-3 rounded-lg px-3 py-2 text-[13px] font-medium transition-colors duration-150",
                    isActive
                      ? "bg-[var(--accent-cyan)]/10 text-[var(--accent-cyan)]"
                      : "text-[var(--text-secondary)] hover:bg-[var(--bg-surface)] hover:text-[var(--text-primary)]",
                  ].join(" ")
                }
              >
                <item.icon className="h-4 w-4 shrink-0" />
                <span className="font-body">{item.label}</span>
              </NavLink>
            ))}
          </div>
        ))}
      </div>

      <div className="border-t border-[var(--border-subtle)] px-5 py-3">
        <p className="font-mono text-[10px] text-[var(--text-muted)]">
          7D Dark Universe
        </p>
      </div>
    </nav>
  );
}
