// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
#![allow(clippy::needless_range_loop)]
use std::fmt;

const DIM: usize = 7;
const PHYSICAL_DIM: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DarkDimension {
    E = 3,
    S = 4,
    T = 5,
    Mu = 6,
}

impl DarkDimension {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn all() -> &'static [DarkDimension; 4] {
        &[
            DarkDimension::E,
            DarkDimension::S,
            DarkDimension::T,
            DarkDimension::Mu,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            DarkDimension::E => "Energy",
            DarkDimension::S => "Space",
            DarkDimension::T => "Time",
            DarkDimension::Mu => "Mu (mass coupling)",
        }
    }
}

impl fmt::Display for DarkDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhysicalDimension {
    X = 0,
    Y = 1,
    Z = 2,
}

impl PhysicalDimension {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn all() -> &'static [PhysicalDimension; 3] {
        &[
            PhysicalDimension::X,
            PhysicalDimension::Y,
            PhysicalDimension::Z,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DimensionPhysics {
    pub metric_weight: f64,
    pub propagation_decay: f64,
    pub coupling_strength: f64,
}

impl Default for DimensionPhysics {
    fn default() -> Self {
        Self {
            metric_weight: 1.0,
            propagation_decay: 1.0,
            coupling_strength: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DimensionProfile {
    dims: [DimensionPhysics; DIM],
}

impl Default for DimensionProfile {
    fn default() -> Self {
        Self {
            dims: [DimensionPhysics::default(); DIM],
        }
    }
}

impl DimensionProfile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn physical_physics(&self, pd: PhysicalDimension) -> &DimensionPhysics {
        &self.dims[pd.index()]
    }

    pub fn dark_physics(&self, dd: DarkDimension) -> &DimensionPhysics {
        &self.dims[dd.index()]
    }

    pub fn get(&self, dim: usize) -> &DimensionPhysics {
        &self.dims[dim]
    }

    pub fn set(&mut self, dim: usize, physics: DimensionPhysics) {
        if dim < DIM {
            self.dims[dim] = physics;
        }
    }

    pub fn set_physical(&mut self, pd: PhysicalDimension, physics: DimensionPhysics) {
        self.dims[pd.index()] = physics;
    }

    pub fn set_dark(&mut self, dd: DarkDimension, physics: DimensionPhysics) {
        self.dims[dd.index()] = physics;
    }

    pub fn metric_weights(&self) -> [f64; DIM] {
        let mut w = [0.0; DIM];
        for (i, d) in self.dims.iter().enumerate() {
            w[i] = d.metric_weight;
        }
        w
    }

    pub fn propagation_decays(&self) -> [f64; DIM] {
        let mut d = [0.0; DIM];
        for (i, dim) in self.dims.iter().enumerate() {
            d[i] = dim.propagation_decay;
        }
        d
    }

    pub fn dark_anisotropy() -> Self {
        let mut profile = Self::default();
        profile.set_dark(
            DarkDimension::E,
            DimensionPhysics {
                metric_weight: 1.2,
                propagation_decay: 0.85,
                coupling_strength: 0.3,
            },
        );
        profile.set_dark(
            DarkDimension::S,
            DimensionPhysics {
                metric_weight: 1.0,
                propagation_decay: 0.72,
                coupling_strength: 0.5,
            },
        );
        profile.set_dark(
            DarkDimension::T,
            DimensionPhysics {
                metric_weight: 0.8,
                propagation_decay: 0.60,
                coupling_strength: 0.7,
            },
        );
        profile.set_dark(
            DarkDimension::Mu,
            DimensionPhysics {
                metric_weight: 1.5,
                propagation_decay: 0.90,
                coupling_strength: 0.2,
            },
        );
        profile
    }

    pub fn from_emotion_weights(weights: [f64; DIM], base: &DimensionProfile) -> Self {
        let mut profile = base.clone();
        for i in 0..DIM {
            let w = weights[i].max(0.1);
            let base_dp = base.dims[i];
            profile.dims[i] = DimensionPhysics {
                metric_weight: base_dp.metric_weight * w,
                propagation_decay: base_dp.propagation_decay * (1.0 / w).min(2.0),
                coupling_strength: base_dp.coupling_strength,
            };
        }
        profile
    }

    pub fn weighted_distance_sq(&self, a: &[f64; DIM], b: &[f64; DIM]) -> f64 {
        let mut sum = 0.0;
        for i in 0..DIM {
            let d = a[i] - b[i];
            sum += self.dims[i].metric_weight * d * d;
        }
        sum
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricTensor {
    g: [[f64; DIM]; DIM],
}

impl Default for MetricTensor {
    fn default() -> Self {
        let mut g = [[0.0; DIM]; DIM];
        for i in 0..DIM {
            g[i][i] = 1.0;
        }
        Self { g }
    }
}

impl MetricTensor {
    pub fn euclidean() -> Self {
        Self::default()
    }

    pub fn from_profile(profile: &DimensionProfile) -> Self {
        let mut g = [[0.0; DIM]; DIM];
        for i in 0..DIM {
            g[i][i] = profile.get(i).metric_weight;
        }
        Self { g }
    }

    pub fn from_profile_with_coupling(profile: &DimensionProfile) -> Self {
        let mut g = [[0.0; DIM]; DIM];
        for i in 0..DIM {
            g[i][i] = profile.get(i).metric_weight;
        }
        for dd in DarkDimension::all() {
            let idx = dd.index();
            let cs = profile.get(idx).coupling_strength;
            for pd in PhysicalDimension::all() {
                let pidx = pd.index();
                g[idx][pidx] = cs * 0.5;
                g[pidx][idx] = cs * 0.5;
            }
        }
        let result = Self { g };
        if !result.is_diagonally_dominant() {
            tracing::warn!("MetricTensor: coupling produced non-diagonally-dominant matrix");
        }
        result
    }

    pub fn curved(local_energy_density: f64, base: &MetricTensor) -> Self {
        let curvature_factor = 1.0 + 0.1 * local_energy_density.min(100.0);
        let mut g = base.g;
        for i in 0..DIM {
            g[i][i] *= curvature_factor;
        }
        Self { g }
    }

    pub fn get(&self, i: usize, j: usize) -> f64 {
        if i < DIM && j < DIM {
            self.g[i][j]
        } else {
            0.0
        }
    }

    pub fn is_diagonally_dominant(&self) -> bool {
        for i in 0..DIM {
            let diag = self.g[i][i];
            if diag <= 0.0 {
                return false;
            }
            let off_diag_sum: f64 = (0..DIM)
                .filter(|&j| j != i)
                .map(|j| self.g[i][j].abs())
                .sum();
            if off_diag_sum >= diag {
                return false;
            }
        }
        true
    }

    pub fn distance_sq(&self, a: &[f64; DIM], b: &[f64; DIM]) -> f64 {
        let mut sum = 0.0;
        for i in 0..DIM {
            for j in 0..DIM {
                let di = a[i] - b[i];
                let dj = a[j] - b[j];
                sum += self.g[i][j] * di * dj;
            }
        }
        sum
    }

    pub fn scalar_curvature(&self, energy_fields: &[([f64; DIM], f64)]) -> f64 {
        if energy_fields.len() < 2 {
            return 0.0;
        }
        let mut total_tidal = 0.0;
        let mut count = 0usize;
        for i in 0..energy_fields.len() {
            for j in (i + 1)..energy_fields.len() {
                let (pos_i, energy_i) = energy_fields[i];
                let (pos_j, energy_j) = energy_fields[j];
                let dist = self.distance_sq(&pos_i, &pos_j).sqrt().max(0.001);
                let tidal = (energy_i - energy_j).abs() / dist;
                total_tidal += tidal;
                count += 1;
            }
        }
        if count > 0 {
            total_tidal / count as f64
        } else {
            0.0
        }
    }

    pub fn geodesic_step(
        &self,
        current: &[f64; DIM],
        target: &[f64; DIM],
        step_size: f64,
    ) -> [f64; DIM] {
        let mut grad = [0.0; DIM];
        for i in 0..DIM {
            for j in 0..DIM {
                grad[i] += self.g[i][j] * (target[j] - current[j]);
            }
        }
        let grad_norm: f64 = grad.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-15);
        let mut result = *current;
        for i in 0..DIM {
            result[i] += step_size * grad[i] / grad_norm;
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CouplingMatrix {
    c: [[f64; DIM]; DIM],
}

impl Default for CouplingMatrix {
    fn default() -> Self {
        Self {
            c: [[0.0; DIM]; DIM],
        }
    }
}

impl CouplingMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_profile(profile: &DimensionProfile) -> Self {
        let mut c = [[0.0; DIM]; DIM];
        for dd in DarkDimension::all() {
            let idx = dd.index();
            let cs = profile.get(idx).coupling_strength;
            for pd in PhysicalDimension::all() {
                let pidx = pd.index();
                c[idx][pidx] = cs;
                c[pidx][idx] = cs;
            }
        }
        Self { c }
    }

    pub fn identity_coupling() -> Self {
        let mut c = [[0.0; DIM]; DIM];
        for i in 0..DIM {
            c[i][i] = 1.0;
        }
        Self { c }
    }

    pub fn get(&self, i: usize, j: usize) -> f64 {
        if i < DIM && j < DIM {
            self.c[i][j]
        } else {
            0.0
        }
    }
    pub fn set(&mut self, i: usize, j: usize, val: f64) {
        if i < DIM && j < DIM {
            self.c[i][j] = val;
            self.c[j][i] = val;
        }
    }

    pub fn coupled_flow(&self, dims: &mut [f64; DIM], from_dim: usize, amount: f64) -> f64 {
        if from_dim >= DIM || amount <= 0.0 {
            return 0.0;
        }
        let available = dims[from_dim];
        let actual = amount.min(available);
        dims[from_dim] -= actual;
        let mut total_coupled = 0.0;
        let mut coupled_count = 0usize;
        for j in 0..DIM {
            if j != from_dim && self.c[from_dim][j] > 0.0 {
                coupled_count += 1;
            }
        }
        if coupled_count > 0 {
            let per_coupled = actual * 0.1;
            for j in 0..DIM {
                if j != from_dim && self.c[from_dim][j] > 0.0 {
                    let transfer = per_coupled * self.c[from_dim][j];
                    dims[j] += transfer;
                    total_coupled += transfer;
                }
            }
        }
        if total_coupled > actual {
            let excess = total_coupled - actual;
            if coupled_count > 0 {
                let per_correction = excess / coupled_count as f64;
                let mut corrected = 0.0;
                for j in 0..DIM {
                    if j != from_dim && self.c[from_dim][j] > 0.0 {
                        let deduct = per_correction.min(dims[j]);
                        dims[j] -= deduct;
                        corrected += deduct;
                    }
                }
                dims[from_dim] += excess - corrected;
            }
            total_coupled = actual;
        }
        let remainder = actual - total_coupled;
        dims[from_dim] += remainder;
        remainder
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseTransitionConfig {
    pub sharpness: f64,
    pub threshold: f64,
    pub fluctuation_amplitude: f64,
    pub temperature: f64,
}

impl Default for PhaseTransitionConfig {
    fn default() -> Self {
        Self {
            sharpness: f64::INFINITY,
            threshold: 0.5,
            fluctuation_amplitude: 0.0,
            temperature: 0.0,
        }
    }
}

impl PhaseTransitionConfig {
    pub fn hard(threshold: f64) -> Self {
        Self {
            sharpness: f64::INFINITY,
            threshold,
            fluctuation_amplitude: 0.0,
            temperature: 0.0,
        }
    }

    pub fn sigmoid(threshold: f64, sharpness: f64) -> Self {
        Self {
            sharpness,
            threshold,
            fluctuation_amplitude: 0.0,
            temperature: 0.0,
        }
    }

    pub fn thermal(threshold: f64, sharpness: f64, temperature: f64) -> Self {
        Self {
            sharpness,
            threshold,
            fluctuation_amplitude: 0.02 * temperature,
            temperature,
        }
    }

    pub fn manifestation_probability(&self, ratio: f64) -> f64 {
        if self.sharpness.is_infinite() || self.sharpness <= 0.0 {
            if ratio >= self.threshold {
                1.0
            } else {
                0.0
            }
        } else {
            let x = self.sharpness * (ratio - self.threshold);
            1.0 / (1.0 + (-x).exp())
        }
    }

    pub fn is_manifested(&self, ratio: f64) -> bool {
        if self.fluctuation_amplitude > 0.0 {
            let noise = self.fluctuation_amplitude * (simple_hash_f64(ratio) * 2.0 - 1.0);
            self.manifestation_probability(ratio + noise) > 0.5
        } else {
            self.manifestation_probability(ratio) > 0.5
        }
    }
}

fn simple_hash_f64(x: f64) -> f64 {
    let bits = x.to_bits();
    let mut h = bits.wrapping_mul(0x9e3779b97f4a7c15);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    (h & 0xFFFF_FFFF_u64) as f64 / 0xFFFF_FFFF_u64 as f64
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionMatrix {
    m: [[f64; DIM]; PHYSICAL_DIM],
}

impl Default for ProjectionMatrix {
    fn default() -> Self {
        let mut m = [[0.0; DIM]; PHYSICAL_DIM];
        for i in 0..PHYSICAL_DIM {
            m[i][i] = 1.0;
        }
        Self { m }
    }
}

impl ProjectionMatrix {
    pub fn orthogonal() -> Self {
        Self::default()
    }

    pub fn with_dark_mixing(mix_angle: f64) -> Self {
        let mut m = [[0.0; DIM]; PHYSICAL_DIM];
        let cos_a = mix_angle.cos();
        let sin_a = mix_angle.sin();
        m[0][0] = cos_a;
        m[0][3] = sin_a * 0.5;
        m[1][1] = cos_a;
        m[1][4] = sin_a * 0.3;
        m[2][2] = cos_a;
        m[2][5] = sin_a * 0.4;
        Self { m }
    }

    pub fn from_rotation(axis_i: usize, axis_j: usize, angle: f64) -> Self {
        let mut m = [[0.0; DIM]; PHYSICAL_DIM];
        for i in 0..PHYSICAL_DIM {
            m[i][i] = 1.0;
        }
        if axis_i < PHYSICAL_DIM && axis_j < DIM && axis_i != axis_j {
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            m[axis_i][axis_i] = cos_a;
            m[axis_i][axis_j] = -sin_a;
            if axis_j < PHYSICAL_DIM {
                m[axis_j][axis_i] = sin_a;
                m[axis_j][axis_j] = cos_a;
            }
        }
        Self { m }
    }

    pub fn project(&self, coords_7d: &[f64; DIM]) -> [f64; PHYSICAL_DIM] {
        let mut result = [0.0; PHYSICAL_DIM];
        for i in 0..PHYSICAL_DIM {
            for j in 0..DIM {
                result[i] += self.m[i][j] * coords_7d[j];
            }
        }
        result
    }

    pub fn get(&self, i: usize, j: usize) -> f64 {
        if i < PHYSICAL_DIM && j < DIM {
            self.m[i][j]
        } else {
            0.0
        }
    }

    pub fn project_energy(&self, energy: &[f64; DIM]) -> [f64; PHYSICAL_DIM] {
        let mut result = [0.0; PHYSICAL_DIM];
        for i in 0..PHYSICAL_DIM {
            for j in 0..DIM {
                result[i] += self.m[i][j] * energy[j];
            }
        }
        result
    }

    pub fn physical_energy_ratio(&self, energy: &[f64; DIM]) -> f64 {
        let projected = self.project_energy(energy);
        let proj_total: f64 = projected.iter().sum();
        let full_total: f64 = energy.iter().sum();
        if full_total <= 0.0 {
            0.0
        } else {
            proj_total.max(0.0) / full_total
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniversePhysics {
    pub profile: DimensionProfile,
    pub metric: MetricTensor,
    pub coupling: CouplingMatrix,
    pub phase: PhaseTransitionConfig,
    pub projection: ProjectionMatrix,
}

impl Default for UniversePhysics {
    fn default() -> Self {
        Self {
            profile: DimensionProfile::default(),
            metric: MetricTensor::euclidean(),
            coupling: CouplingMatrix::new(),
            phase: PhaseTransitionConfig::hard(0.5),
            projection: ProjectionMatrix::orthogonal(),
        }
    }
}

impl UniversePhysics {
    pub fn flat() -> Self {
        Self::default()
    }

    pub fn rich() -> Self {
        let profile = DimensionProfile::dark_anisotropy();
        let metric = MetricTensor::from_profile_with_coupling(&profile);
        let coupling = CouplingMatrix::from_profile(&profile);
        Self {
            profile,
            metric,
            coupling,
            phase: PhaseTransitionConfig::thermal(0.5, 12.0, 1.0),
            projection: ProjectionMatrix::with_dark_mixing(0.3),
        }
    }

    pub fn with_profile(profile: DimensionProfile) -> Self {
        let metric = MetricTensor::from_profile_with_coupling(&profile);
        let coupling = CouplingMatrix::from_profile(&profile);
        Self {
            profile,
            metric,
            coupling,
            phase: PhaseTransitionConfig::default(),
            projection: ProjectionMatrix::orthogonal(),
        }
    }

    pub fn steered_by_emotion(base: &UniversePhysics, weights: [f64; DIM]) -> Self {
        let profile = DimensionProfile::from_emotion_weights(weights, &base.profile);
        let metric = MetricTensor::from_profile_with_coupling(&profile);
        let coupling = CouplingMatrix::from_profile(&profile);
        Self {
            profile,
            metric,
            coupling,
            phase: base.phase,
            projection: base.projection.clone(),
        }
    }

    pub fn weighted_distance_sq(&self, a: &[f64; DIM], b: &[f64; DIM]) -> f64 {
        self.metric.distance_sq(a, b)
    }

    pub fn is_manifested(&self, ratio: f64) -> bool {
        self.phase.is_manifested(ratio)
    }

    pub fn project_to_physical(&self, coords_7d: &[f64; DIM]) -> [f64; PHYSICAL_DIM] {
        self.projection.project(coords_7d)
    }

    pub fn geodesic_step(
        &self,
        current: &[f64; DIM],
        target: &[f64; DIM],
        step_size: f64,
    ) -> [f64; DIM] {
        self.metric.geodesic_step(current, target, step_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_physics_is_flat() {
        let p = UniversePhysics::flat();
        assert_eq!(p.phase.sharpness, f64::INFINITY);
        assert_eq!(p.phase.threshold, 0.5);
        assert!(p.coupling.get(0, 3) == 0.0);
    }

    #[test]
    fn dark_dimensions_are_correct() {
        assert_eq!(DarkDimension::E.index(), 3);
        assert_eq!(DarkDimension::S.index(), 4);
        assert_eq!(DarkDimension::T.index(), 5);
        assert_eq!(DarkDimension::Mu.index(), 6);
    }

    #[test]
    fn hard_phase_transition_exact() {
        let pt = PhaseTransitionConfig::hard(0.5);
        assert!(pt.is_manifested(0.5));
        assert!(pt.is_manifested(0.6));
        assert!(!pt.is_manifested(0.49));
    }

    #[test]
    fn sigmoid_phase_transition_smooth() {
        let pt = PhaseTransitionConfig::sigmoid(0.5, 30.0);
        assert!(pt.manifestation_probability(0.5) > 0.49);
        assert!(pt.manifestation_probability(0.5) < 0.51);
        assert!(pt.manifestation_probability(0.6) > 0.9);
        assert!(pt.manifestation_probability(0.4) < 0.1);
    }

    #[test]
    fn rich_physics_has_coupling() {
        let p = UniversePhysics::rich();
        assert!(p.coupling.get(0, 3) > 0.0);
        assert!(p.coupling.get(0, 4) > 0.0);
    }

    #[test]
    fn metric_tensor_curvature() {
        let base = MetricTensor::euclidean();
        let curved = MetricTensor::curved(10.0, &base);
        assert!(curved.get(0, 0) > base.get(0, 0));
    }

    #[test]
    fn metric_distance_anisotropic() {
        let profile = DimensionProfile::dark_anisotropy();
        let metric = MetricTensor::from_profile(&profile);
        let a = [0.0; DIM];
        let b = {
            let mut v = [0.0; DIM];
            v[0] = 1.0;
            v
        };
        let c = {
            let mut v = [0.0; DIM];
            v[3] = 1.0;
            v
        };
        let d_physical = metric.distance_sq(&a, &b);
        let d_dark_e = metric.distance_sq(&a, &c);
        assert!(d_physical > 0.0);
        assert!(d_dark_e > 0.0);
        assert!((d_physical - d_dark_e).abs() > 0.01);
    }

    #[test]
    fn projection_orthogonal_is_identity() {
        let proj = ProjectionMatrix::orthogonal();
        let coords = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let physical = proj.project(&coords);
        assert!((physical[0] - 1.0).abs() < 1e-10);
        assert!((physical[1] - 2.0).abs() < 1e-10);
        assert!((physical[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn projection_with_mixing_sees_dark() {
        let proj = ProjectionMatrix::with_dark_mixing(0.5);
        let coords = [0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.0];
        let physical = proj.project(&coords);
        assert!(physical[0].abs() > 0.1, "dark E should affect physical x");
    }

    #[test]
    fn coupling_matrix_flow_preserves_direction() {
        let profile = DimensionProfile::dark_anisotropy();
        let coupling = CouplingMatrix::from_profile(&profile);
        let mut dims = [100.0; DIM];
        let net = coupling.coupled_flow(&mut dims, 0, 10.0);
        assert!(dims[0] < 100.0);
        assert!(net > 0.0);
    }

    #[test]
    fn geodesic_moves_toward_target() {
        let metric = MetricTensor::euclidean();
        let current = [0.0; DIM];
        let target = [10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let step = metric.geodesic_step(&current, &target, 0.5);
        assert!(step[0] > 0.0);
    }

    #[test]
    fn rich_physics_integrated() {
        let physics = UniversePhysics::rich();
        let a = [0.0; DIM];
        let b = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let dist = physics.weighted_distance_sq(&a, &b);
        assert!(dist > 0.0);
        assert!(physics.is_manifested(0.7));
    }

    #[test]
    fn scalar_curvature_with_gradient() {
        let metric = MetricTensor::euclidean();
        let fields = vec![
            ([0.0; DIM], 100.0),
            ([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 50.0),
        ];
        let r = metric.scalar_curvature(&fields);
        assert!(r > 0.0);
    }

    #[test]
    fn scalar_curvature_empty_is_zero() {
        let metric = MetricTensor::euclidean();
        let r = metric.scalar_curvature(&[]);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn thermal_fluctuation_varies() {
        let pt = PhaseTransitionConfig::thermal(0.5, 10.0, 1.0);
        assert!(pt.fluctuation_amplitude > 0.0);
    }

    #[test]
    fn physical_dimensions_correct() {
        assert_eq!(PhysicalDimension::X.index(), 0);
        assert_eq!(PhysicalDimension::Y.index(), 1);
        assert_eq!(PhysicalDimension::Z.index(), 2);
    }
}
