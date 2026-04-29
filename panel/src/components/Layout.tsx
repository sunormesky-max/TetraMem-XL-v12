import type { ReactNode } from 'react'
import { Suspense } from 'react'
import Navbar from './Navbar'
import Header from './Header'
import Footer from './Footer'

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <div className="flex min-h-[100dvh] bg-[var(--bg-void)]">
      {/* Sidebar */}
      <Navbar />

      {/* Main Content */}
      <div
        className="flex flex-1 flex-col"
        style={{ marginLeft: 'var(--sidebar-width)' }}
      >
        <Header />

        <main className="relative flex-1 overflow-y-auto">
          <Suspense
            fallback={
              <div className="flex h-full items-center justify-center">
                <div className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--accent-cyan)] border-t-transparent" />
              </div>
            }
          >
            {children}
          </Suspense>
        </main>

        <Footer />
      </div>
    </div>
  )
}
