import { useRef, useMemo } from 'react'
import { Canvas, useFrame } from '@react-three/fiber'
import type { Mesh } from 'three'
import * as THREE from 'three'

const PARTICLE_COUNT = 300
const CONNECTION_THRESHOLD = 2.5
const DIMENSION_COLORS = [
  '#00E5FF', '#00B0FF', '#2979FF', '#FFD600',
  '#FF6D00', '#F50057', '#D500F9',
]

interface ParticleFieldProps {
  mouseRef: React.MutableRefObject<{ x: number; y: number }>
}

function ParticleField({ mouseRef }: ParticleFieldProps) {
  const groupRef = useRef<THREE.Group>(null)
  const meshRefs = useRef<Mesh[]>([])

  const particles = useMemo(() => {
    return Array.from({ length: PARTICLE_COUNT }, (_, i) => ({
      id: i,
      position: [
        (Math.random() - 0.5) * 20,
        (Math.random() - 0.5) * 12,
        (Math.random() - 0.5) * 8,
      ] as [number, number, number],
      color: DIMENSION_COLORS[i % DIMENSION_COLORS.length],
      phase: Math.random() * Math.PI * 2,
      speed: 0.2 + Math.random() * 0.4,
      amplitude: 0.3 + Math.random() * 0.7,
    }))
  }, [])

  useFrame((state) => {
    const t = state.clock.elapsedTime
    const mx = mouseRef.current.x * 0.5
    const my = mouseRef.current.y * 0.5

    if (groupRef.current) {
      groupRef.current.rotation.y = t * 0.02 + mx * 0.1
      groupRef.current.rotation.x = t * 0.01 + my * 0.05
    }

    meshRefs.current.forEach((mesh, i) => {
      if (!mesh) return
      const p = particles[i]
      const baseX = p.position[0]
      const baseY = p.position[1]
      const baseZ = p.position[2]

      const sineX = Math.sin(t * p.speed + p.phase) * p.amplitude * 0.3
      const sineY = Math.cos(t * p.speed * 0.7 + p.phase) * p.amplitude * 0.2
      const sineZ = Math.sin(t * p.speed * 0.5 + p.phase) * p.amplitude * 0.15

      mesh.position.x = baseX + sineX
      mesh.position.y = baseY + sineY
      mesh.position.z = baseZ + sineZ
      mesh.rotation.x = t * 0.5 + p.phase
      mesh.rotation.y = t * 0.3 + p.phase
    })
  })

  return (
    <group ref={groupRef}>
      {particles.map((p, i) => (
        <mesh
          key={p.id}
          ref={(el) => {
            if (el) meshRefs.current[i] = el
          }}
          position={p.position}
        >
          <tetrahedronGeometry args={[0.08, 0]} />
          <meshBasicMaterial
            color={p.color}
            transparent
            opacity={0.7}
            wireframe={false}
          />
        </mesh>
      ))}

      {/* Connection lines */}
      <ConnectionLines particles={particles} meshRefs={meshRefs} />
    </group>
  )
}

interface ConnectionLinesProps {
  particles: Array<{
    id: number
    position: [number, number, number]
    color: string
  }>
  meshRefs: React.MutableRefObject<Mesh[]>
}

function ConnectionLines({ particles, meshRefs }: ConnectionLinesProps) {
  const linesRef = useRef<THREE.Group>(null)

  const lineGeometries = useMemo(() => {
    const geoms: THREE.BufferGeometry[] = []
    for (let i = 0; i < particles.length; i++) {
      for (let j = i + 1; j < particles.length; j++) {
        const dx = particles[i].position[0] - particles[j].position[0]
        const dy = particles[i].position[1] - particles[j].position[1]
        const dz = particles[i].position[2] - particles[j].position[2]
        const dist = Math.sqrt(dx * dx + dy * dy + dz * dz)
        if (dist < CONNECTION_THRESHOLD) {
          const geometry = new THREE.BufferGeometry()
          const vertices = new Float32Array([
            particles[i].position[0],
            particles[i].position[1],
            particles[i].position[2],
            particles[j].position[0],
            particles[j].position[1],
            particles[j].position[2],
          ])
          geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3))
          geometry.userData = { i, j, dist }
          geoms.push(geometry)
        }
      }
    }
    return geoms
  }, [particles])

  useFrame(() => {
    if (!linesRef.current) return
    const children = linesRef.current.children as THREE.Line[]
    children.forEach((line) => {
      const geom = line.geometry as THREE.BufferGeometry
      const { i, j } = geom.userData as { i: number; j: number }
      const meshI = meshRefs.current[i]
      const meshJ = meshRefs.current[j]
      if (!meshI || !meshJ) return

      const positions = geom.attributes.position.array as Float32Array
      positions[0] = meshI.position.x
      positions[1] = meshI.position.y
      positions[2] = meshI.position.z
      positions[3] = meshJ.position.x
      positions[4] = meshJ.position.y
      positions[5] = meshJ.position.z
      geom.attributes.position.needsUpdate = true

      const dx = meshI.position.x - meshJ.position.x
      const dy = meshI.position.y - meshJ.position.y
      const dz = meshI.position.z - meshJ.position.z
      const dist = Math.sqrt(dx * dx + dy * dy + dz * dz)
      const opacity = Math.max(0, 1 - dist / CONNECTION_THRESHOLD) * 0.15
      const mat = line.material as THREE.LineBasicMaterial
      mat.opacity = opacity
      mat.transparent = true
    })
  })

  return (
    <group ref={linesRef}>
      {lineGeometries.map((geom, idx) => (
        <primitive key={idx} object={new THREE.Line(geom, new THREE.LineBasicMaterial({ color: '#00E5FF', transparent: true, opacity: 0.05 }))} />
      ))}
    </group>
  )
}

export default function ParticleBackground() {
  const mouseRef = useRef({ x: 0, y: 0 })

  const handleMouseMove = (e: React.MouseEvent) => {
    mouseRef.current.x = (e.clientX / window.innerWidth - 0.5) * 2
    mouseRef.current.y = (e.clientY / window.innerHeight - 0.5) * 2
  }

  return (
    <div
      className="fixed inset-0 z-0"
      style={{ background: 'var(--bg-deep)' }}
      onMouseMove={handleMouseMove}
    >
      <div
        className="absolute inset-0 z-10"
        style={{ background: 'var(--gradient-hero)' }}
      />
      <Canvas
        camera={{ position: [0, 0, 8], fov: 60 }}
        style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%' }}
        gl={{ antialias: true, alpha: true }}
        dpr={[1, 1.5]}
      >
        <ambientLight intensity={0.3} />
        <ParticleField mouseRef={mouseRef} />
      </Canvas>
    </div>
  )
}
