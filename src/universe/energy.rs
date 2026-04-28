use std::fmt;

const DIM: usize = 7;
const PHYSICAL_DIM: usize = 3;
const DARK_DIM: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnergyError {
    InsufficientEnergy { requested: f64, available: f64 },
    NegativeAmount,
    InvalidDimension { dim: usize },
    InvalidRatio { ratio: f64 },
    AlreadyOccupied,
}

impl fmt::Display for EnergyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnergyError::InsufficientEnergy { requested, available } => {
                write!(f, "insufficient energy: need {:.4}, have {:.4}", requested, available)
            }
            EnergyError::NegativeAmount => write!(f, "energy amount must be non-negative"),
            EnergyError::InvalidDimension { dim } => {
                write!(f, "dimension {} out of range [0, {})", dim, DIM)
            }
            EnergyError::InvalidRatio { ratio } => {
                write!(f, "ratio {} out of range [0.0, 1.0]", ratio)
            }
            EnergyError::AlreadyOccupied => write!(f, "position already occupied"),
        }
    }
}

impl std::error::Error for EnergyError {}

#[derive(Debug, Clone, PartialEq)]
pub struct EnergyField {
    dims: [f64; DIM],
}

impl EnergyField {
    pub fn zero() -> Self {
        Self { dims: [0.0; DIM] }
    }

    pub fn uniform(total: f64) -> Self {
        assert!(total >= 0.0);
        let per_dim = total / DIM as f64;
        Self { dims: [per_dim; DIM] }
    }

    pub fn with_physical_bias(total: f64, physical_ratio: f64) -> Self {
        assert!(total >= 0.0);
        assert!(
            (0.0..=1.0).contains(&physical_ratio),
            "physical_ratio must be in [0, 1]"
        );
        let phys_total = total * physical_ratio;
        let dark_total = total * (1.0 - physical_ratio);
        let per_phys = if PHYSICAL_DIM > 0 {
            phys_total / PHYSICAL_DIM as f64
        } else {
            0.0
        };
        let per_dark = if DARK_DIM > 0 {
            dark_total / DARK_DIM as f64
        } else {
            0.0
        };
        Self {
            dims: [
                per_phys,
                per_phys,
                per_phys,
                per_dark,
                per_dark,
                per_dark,
                per_dark,
            ],
        }
    }

    pub fn from_dims(dims: [f64; DIM]) -> Self {
        assert!(dims.iter().all(|&d| d >= 0.0), "all dimensions must be non-negative");
        Self { dims }
    }

    pub fn total(&self) -> f64 {
        self.dims.iter().sum()
    }

    pub fn physical(&self) -> f64 {
        self.dims[0] + self.dims[1] + self.dims[2]
    }

    pub fn dark(&self) -> f64 {
        self.dims[3] + self.dims[4] + self.dims[5] + self.dims[6]
    }

    pub fn manifestation_ratio(&self) -> f64 {
        let t = self.total();
        if t <= 0.0 {
            0.0
        } else {
            self.physical() / t
        }
    }

    pub fn dim(&self, i: usize) -> f64 {
        self.dims[i]
    }

    pub fn dims(&self) -> &[f64; DIM] {
        &self.dims
    }

    pub fn dims_mut(&mut self) -> &mut [f64; DIM] {
        &mut self.dims
    }

    pub fn is_empty(&self) -> bool {
        self.total() <= 0.0
    }

    pub fn is_manifested(&self, threshold: f64) -> bool {
        self.total() > 0.0 && self.manifestation_ratio() >= threshold
    }

    pub fn flow(&mut self, from_dim: usize, to_dim: usize, amount: f64) -> Result<(), EnergyError> {
        if from_dim >= DIM {
            return Err(EnergyError::InvalidDimension { dim: from_dim });
        }
        if to_dim >= DIM {
            return Err(EnergyError::InvalidDimension { dim: to_dim });
        }
        if amount < 0.0 {
            return Err(EnergyError::NegativeAmount);
        }
        if self.dims[from_dim] < amount - 1e-15 {
            return Err(EnergyError::InsufficientEnergy {
                requested: amount,
                available: self.dims[from_dim],
            });
        }
        self.dims[from_dim] -= amount;
        self.dims[to_dim] += amount;
        Ok(())
    }

    pub fn flow_physical_to_dark(&mut self, amount: f64) -> Result<(), EnergyError> {
        let per_phys = amount / PHYSICAL_DIM as f64;
        let per_dark = amount / DARK_DIM as f64;
        for i in 0..PHYSICAL_DIM {
            if self.dims[i] < per_phys - 1e-15 {
                return Err(EnergyError::InsufficientEnergy {
                    requested: per_phys,
                    available: self.dims[i],
                });
            }
        }
        for i in 0..PHYSICAL_DIM {
            self.dims[i] -= per_phys;
        }
        for i in PHYSICAL_DIM..DIM {
            self.dims[i] += per_dark;
        }
        Ok(())
    }

    pub fn flow_dark_to_physical(&mut self, amount: f64) -> Result<(), EnergyError> {
        let per_dark = amount / DARK_DIM as f64;
        let per_phys = amount / PHYSICAL_DIM as f64;
        for i in PHYSICAL_DIM..DIM {
            if self.dims[i] < per_dark - 1e-15 {
                return Err(EnergyError::InsufficientEnergy {
                    requested: per_dark,
                    available: self.dims[i],
                });
            }
        }
        for i in PHYSICAL_DIM..DIM {
            self.dims[i] -= per_dark;
        }
        for i in 0..PHYSICAL_DIM {
            self.dims[i] += per_phys;
        }
        Ok(())
    }

    pub fn absorb(&mut self, other: &EnergyField) {
        for i in 0..DIM {
            self.dims[i] += other.dims[i];
        }
    }

    pub fn split_ratio(&mut self, ratio: f64) -> Result<EnergyField, EnergyError> {
        if ratio < 0.0 || ratio > 1.0 {
            return Err(EnergyError::InvalidRatio { ratio });
        }
        let mut taken = [0.0f64; DIM];
        for i in 0..DIM {
            taken[i] = self.dims[i] * ratio;
            self.dims[i] -= taken[i];
        }
        Ok(EnergyField { dims: taken })
    }

    pub fn split_amount(&mut self, amount: f64) -> Result<EnergyField, EnergyError> {
        if amount < 0.0 {
            return Err(EnergyError::NegativeAmount);
        }
        let total = self.total();
        if amount > total + 1e-15 {
            return Err(EnergyError::InsufficientEnergy {
                requested: amount,
                available: total,
            });
        }
        if total <= 0.0 {
            return Ok(EnergyField::zero());
        }
        self.split_ratio(amount / total)
    }

    pub fn verify_integrity(&self) -> bool {
        self.dims.iter().all(|&d| d >= -1e-10)
    }
}

impl fmt::Display for EnergyField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:.2},{:.2},{:.2} | {:.2},{:.2},{:.2},{:.2}] T={:.2} M={:.1}%",
            self.dims[0],
            self.dims[1],
            self.dims[2],
            self.dims[3],
            self.dims[4],
            self.dims[5],
            self.dims[6],
            self.total(),
            self.manifestation_ratio() * 100.0
        )
    }
}

#[derive(Clone)]
pub struct EnergyPool {
    total: f64,
    allocated: f64,
}

impl EnergyPool {
    pub fn new(total_budget: f64) -> Self {
        assert!(total_budget > 0.0, "total energy must be positive");
        Self {
            total: total_budget,
            allocated: 0.0,
        }
    }

    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn allocated(&self) -> f64 {
        self.allocated
    }

    pub fn available(&self) -> f64 {
        self.total - self.allocated
    }

    pub fn utilization(&self) -> f64 {
        if self.total == 0.0 {
            return 0.0;
        }
        self.allocated / self.total
    }

    pub fn allocate(&mut self, amount: f64) -> Result<f64, EnergyError> {
        if amount < 0.0 {
            return Err(EnergyError::NegativeAmount);
        }
        if amount > self.available() + 1e-15 {
            return Err(EnergyError::InsufficientEnergy {
                requested: amount,
                available: self.available(),
            });
        }
        self.allocated += amount;
        Ok(amount)
    }

    pub fn release(&mut self, amount: f64) {
        self.allocated = (self.allocated - amount).max(0.0);
    }

    pub fn release_field(&mut self, field: &EnergyField) {
        self.release(field.total());
    }

    pub fn verify_conservation(&self) -> bool {
        (self.allocated + self.available() - self.total).abs() < 1e-10
    }

    pub fn expand(&mut self, additional: f64) {
        assert!(additional > 0.0, "expansion amount must be positive");
        self.total += additional;
    }
}

impl fmt::Display for EnergyPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pool(total={:.2}, used={:.2}, free={:.2}, util={:.1}%)",
            self.total,
            self.allocated,
            self.available(),
            self.utilization() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_field() {
        let f = EnergyField::zero();
        assert_eq!(f.total(), 0.0);
        assert_eq!(f.physical(), 0.0);
        assert_eq!(f.dark(), 0.0);
        assert_eq!(f.manifestation_ratio(), 0.0);
        assert!(!f.is_manifested(0.5));
    }

    #[test]
    fn uniform_distribution() {
        let f = EnergyField::uniform(70.0);
        assert!((f.total() - 70.0).abs() < 1e-10);
        assert!((f.dim(0) - 10.0).abs() < 1e-10);
        assert!((f.physical() - 30.0).abs() < 1e-10);
        assert!((f.dark() - 40.0).abs() < 1e-10);
        assert!((f.manifestation_ratio() - 3.0 / 7.0).abs() < 1e-10);
        assert!(f.verify_integrity());
    }

    #[test]
    fn physical_biased() {
        let f = EnergyField::with_physical_bias(100.0, 0.8);
        assert!((f.total() - 100.0).abs() < 1e-10);
        assert!((f.physical() - 80.0).abs() < 1e-10);
        assert!((f.dark() - 20.0).abs() < 1e-10);
        assert!((f.manifestation_ratio() - 0.8).abs() < 1e-10);
        assert!(f.is_manifested(0.5));
        assert!(!f.is_manifested(0.9));
        assert!(f.verify_integrity());
    }

    #[test]
    fn from_dims() {
        let f = EnergyField::from_dims([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        assert!((f.total() - 28.0).abs() < 1e-10);
        assert!((f.physical() - 6.0).abs() < 1e-10);
        assert!((f.dark() - 22.0).abs() < 1e-10);
        assert!(f.verify_integrity());
    }

    #[test]
    fn flow_between_dimensions() {
        let mut f = EnergyField::from_dims([10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        f.flow(0, 3, 3.0).unwrap();
        assert!((f.dim(0) - 7.0).abs() < 1e-10);
        assert!((f.dim(3) - 3.0).abs() < 1e-10);
        assert!((f.total() - 10.0).abs() < 1e-10);
        assert!(f.verify_integrity());
    }

    #[test]
    fn flow_preserves_total() {
        let mut f = EnergyField::uniform(70.0);
        let original_total = f.total();
        for from in 0..7 {
            for to in 0..7 {
                if from != to {
                    f.flow(from, to, 1.0).unwrap();
                }
            }
        }
        assert!((f.total() - original_total).abs() < 1e-10);
        assert!(f.verify_integrity());
    }

    #[test]
    fn flow_insufficient_fails() {
        let mut f = EnergyField::from_dims([2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert!(f.flow(0, 3, 5.0).is_err());
        assert!((f.dim(0) - 2.0).abs() < 1e-10);
        assert!((f.total() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn flow_physical_to_dark() {
        let mut f = EnergyField::with_physical_bias(100.0, 0.9);
        let total_before = f.total();
        f.flow_physical_to_dark(30.0).unwrap();
        assert!((f.total() - total_before).abs() < 1e-10);
        assert!(f.physical() < 90.0);
        assert!(f.dark() > 10.0);
        assert!(f.verify_integrity());
    }

    #[test]
    fn flow_dark_to_physical() {
        let mut f = EnergyField::with_physical_bias(100.0, 0.1);
        let total_before = f.total();
        f.flow_dark_to_physical(20.0).unwrap();
        assert!((f.total() - total_before).abs() < 1e-10);
        assert!(f.physical() > 10.0);
        assert!(f.dark() < 90.0);
        assert!(f.verify_integrity());
    }

    #[test]
    fn split_ratio_preserves_total() {
        let mut f = EnergyField::uniform(70.0);
        let taken = f.split_ratio(0.3).unwrap();
        assert!((f.total() + taken.total() - 70.0).abs() < 1e-10);
        assert!((taken.total() - 21.0).abs() < 1e-10);
        assert!(f.verify_integrity());
        assert!(taken.verify_integrity());
    }

    #[test]
    fn split_amount_preserves_total() {
        let mut f = EnergyField::uniform(70.0);
        let taken = f.split_amount(35.0).unwrap();
        assert!((f.total() + taken.total() - 70.0).abs() < 1e-10);
        assert!((taken.total() - 35.0).abs() < 1e-10);
        assert!(f.verify_integrity());
    }

    #[test]
    fn split_more_than_total_fails() {
        let mut f = EnergyField::uniform(10.0);
        assert!(f.split_amount(20.0).is_err());
        assert!((f.total() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn absorb_adds_per_dimension() {
        let mut a = EnergyField::from_dims([1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0]);
        let b = EnergyField::from_dims([4.0, 5.0, 6.0, 7.0, 0.0, 0.0, 0.0]);
        a.absorb(&b);
        assert!((a.dim(0) - 5.0).abs() < 1e-10);
        assert!((a.dim(3) - 7.0).abs() < 1e-10);
        assert!((a.total() - 28.0).abs() < 1e-10);
    }

    #[test]
    fn manifestation_threshold() {
        let mut f = EnergyField::with_physical_bias(100.0, 0.5);
        assert!(f.is_manifested(0.5));
        assert!(!f.is_manifested(0.6));

        f.flow_physical_to_dark(40.0).unwrap();
        assert!(!f.is_manifested(0.5));
    }

    #[test]
    fn pool_conservation_through_operations() {
        let mut pool = EnergyPool::new(1000.0);

        let a1 = pool.allocate(300.0).unwrap();
        let f1 = EnergyField::with_physical_bias(a1, 0.7);

        let a2 = pool.allocate(200.0).unwrap();
        let f2 = EnergyField::with_physical_bias(a2, 0.3);

        assert!((pool.allocated() - 500.0).abs() < 1e-10);
        assert!(pool.verify_conservation());

        pool.release_field(&f1);
        assert!((pool.allocated() - 200.0).abs() < 1e-10);
        assert!(pool.verify_conservation());

        pool.release_field(&f2);
        assert!((pool.available() - 1000.0).abs() < 1e-10);
        assert!(pool.verify_conservation());
    }

    #[test]
    fn pool_over_allocate_fails() {
        let mut pool = EnergyPool::new(100.0);
        assert!(pool.allocate(200.0).is_err());
        assert!(pool.verify_conservation());
    }

    #[test]
    fn full_cycle_conservation() {
        let mut pool = EnergyPool::new(500.0);
        let mut fields = Vec::new();

        for i in 0..5 {
            let amount = pool.allocate(50.0).unwrap();
            let ratio = 0.3 + (i as f64) * 0.1;
            let f = EnergyField::with_physical_bias(amount, ratio);
            assert!(f.verify_integrity());
            assert!((f.total() - 50.0).abs() < 1e-10);
            fields.push(f);
        }
        assert!(pool.verify_conservation());
        assert!((pool.allocated() - 250.0).abs() < 1e-10);

        for f in &fields {
            pool.release_field(f);
        }
        assert!(pool.verify_conservation());
        assert!((pool.available() - 500.0).abs() < 1e-10);
    }
}
