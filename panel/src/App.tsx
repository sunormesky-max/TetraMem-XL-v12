import { Routes, Route } from 'react-router-dom'
import { lazy } from 'react'
import Layout from './components/Layout'

/* Lazy load all pages for code splitting */
const Dashboard = lazy(() => import('./pages/Home'))
const Universe = lazy(() => import('./pages/Universe'))
const Memory = lazy(() => import('./pages/Memory'))
const Pulse = lazy(() => import('./pages/Pulse'))
const Dream = lazy(() => import('./pages/Dream'))
const Topology = lazy(() => import('./pages/Topology'))
const Regulation = lazy(() => import('./pages/Regulation'))
const ApiPlayground = lazy(() => import('./pages/Api'))
const Cluster = lazy(() => import('./pages/Cluster'))
const Timeline = lazy(() => import('./pages/Timeline'))
const Dark = lazy(() => import('./pages/Dark'))
const Physics = lazy(() => import('./pages/Physics'))
const Semantic = lazy(() => import('./pages/Semantic'))
const Emotion = lazy(() => import('./pages/Emotion'))
const Watchdog = lazy(() => import('./pages/Watchdog'))
const Plugins = lazy(() => import('./pages/Plugins'))

export default function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/universe" element={<Universe />} />
        <Route path="/memory" element={<Memory />} />
        <Route path="/pulse" element={<Pulse />} />
        <Route path="/dream" element={<Dream />} />
        <Route path="/topology" element={<Topology />} />
        <Route path="/regulation" element={<Regulation />} />
        <Route path="/cluster" element={<Cluster />} />
        <Route path="/timeline" element={<Timeline />} />
        <Route path="/dark" element={<Dark />} />
        <Route path="/physics" element={<Physics />} />
        <Route path="/semantic" element={<Semantic />} />
        <Route path="/emotion" element={<Emotion />} />
        <Route path="/watchdog" element={<Watchdog />} />
        <Route path="/plugins" element={<Plugins />} />
        <Route path="/api" element={<ApiPlayground />} />
      </Routes>
    </Layout>
  )
}
