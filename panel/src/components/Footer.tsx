export default function Footer() {
  return (
    <footer className="flex items-center justify-between border-t border-[var(--border-subtle)] bg-[var(--bg-deep)] px-6 py-3">
      <div className="flex items-center gap-3">
        <span className="font-body text-[11px] text-[var(--text-muted)]">
          TetraMem-XL v12.0
        </span>
        <span className="text-[var(--border-subtle)]">|</span>
        <span className="font-mono text-[11px] text-[var(--text-muted)]">
          7D 暗宇宙记忆系统
        </span>
      </div>
      <div className="flex items-center gap-2">
        <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent-green)]" />
        <span className="font-body text-[11px] text-[var(--text-muted)]">
          所有系统正常
        </span>
      </div>
    </footer>
  )
}
