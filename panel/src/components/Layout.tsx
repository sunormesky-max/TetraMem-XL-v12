import { Outlet } from "react-router-dom";
import Navbar from "./Navbar";

export default function Layout() {
  return (
    <div className="flex min-h-screen bg-[var(--bg-void)]">
      <Navbar />
      <main className="flex-1 overflow-auto gradient-hero-bg">
        <Outlet />
      </main>
    </div>
  );
}
