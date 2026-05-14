import { lazy, Suspense } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";

const Home = lazy(() => import("./pages/Home"));
const Memory = lazy(() => import("./pages/Memory"));
const Universe = lazy(() => import("./pages/Universe"));
const Pulse = lazy(() => import("./pages/Pulse"));
const Dream = lazy(() => import("./pages/Dream"));
const Dark = lazy(() => import("./pages/Dark"));
const Topology = lazy(() => import("./pages/Topology"));
const Regulation = lazy(() => import("./pages/Regulation"));
const Cluster = lazy(() => import("./pages/Cluster"));
const Timeline = lazy(() => import("./pages/Timeline"));
const Physics = lazy(() => import("./pages/Physics"));
const Semantic = lazy(() => import("./pages/Semantic"));
const Emotion = lazy(() => import("./pages/Emotion"));
const Watchdog = lazy(() => import("./pages/Watchdog"));
const Plugins = lazy(() => import("./pages/Plugins"));
const ApiPage = lazy(() => import("./pages/Api"));

function Loading() {
  return (
    <div className="flex h-[80dvh] items-center justify-center">
      <div className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--accent-cyan)] border-t-transparent" />
    </div>
  );
}

function NotFound() {
  return (
    <div className="flex h-[80dvh] flex-col items-center justify-center gap-4">
      <h1 className="font-display text-6xl font-bold text-gradient-energy">404</h1>
      <p className="font-body text-[var(--text-muted)]">页面未找到</p>
    </div>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <Suspense fallback={<Loading />}>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<Home />} />
            <Route path="memory" element={<Memory />} />
            <Route path="universe" element={<Universe />} />
            <Route path="pulse" element={<Pulse />} />
            <Route path="dream" element={<Dream />} />
            <Route path="dark" element={<Dark />} />
            <Route path="topology" element={<Topology />} />
            <Route path="regulation" element={<Regulation />} />
            <Route path="cluster" element={<Cluster />} />
            <Route path="timeline" element={<Timeline />} />
            <Route path="physics" element={<Physics />} />
            <Route path="semantic" element={<Semantic />} />
            <Route path="emotion" element={<Emotion />} />
            <Route path="watchdog" element={<Watchdog />} />
            <Route path="plugins" element={<Plugins />} />
            <Route path="api" element={<ApiPage />} />
            <Route path="*" element={<NotFound />} />
          </Route>
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
