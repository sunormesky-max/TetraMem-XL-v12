// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::fmt;

const DIM: usize = 7;

const PHYSICAL_DIM: usize = 3;

const DARK_DIM: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Parity {
    Even,
    Odd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Coord7D {
    basis: [i32; DIM],
    parity: Parity,
}

impl Coord7D {
    pub fn new_even(basis: [i32; DIM]) -> Self {
        Self {
            basis,
            parity: Parity::Even,
        }
    }

    pub fn new_odd(basis: [i32; DIM]) -> Self {
        Self {
            basis,
            parity: Parity::Odd,
        }
    }

    pub fn physical(&self) -> [i32; PHYSICAL_DIM] {
        [self.basis[0], self.basis[1], self.basis[2]]
    }

    pub fn dark(&self) -> [i32; DARK_DIM] {
        [self.basis[3], self.basis[4], self.basis[5], self.basis[6]]
    }

    pub fn as_f64(&self) -> [f64; DIM] {
        let offset = match self.parity {
            Parity::Even => 0.0,
            Parity::Odd => 0.5,
        };
        let mut result = [0.0f64; DIM];
        for (i, b) in self.basis.iter().enumerate() {
            result[i] = *b as f64 + offset;
        }
        result
    }

    pub fn parity(&self) -> Parity {
        self.parity
    }

    pub fn basis(&self) -> [i32; DIM] {
        self.basis
    }

    pub fn is_even(&self) -> bool {
        self.parity == Parity::Even
    }

    pub fn is_odd(&self) -> bool {
        self.parity == Parity::Odd
    }

    pub fn distance_sq(&self, other: &Self) -> f64 {
        let a = self.as_f64();
        let b = other.as_f64();
        let mut sum = 0.0f64;
        for i in 0..DIM {
            let d = a[i] - b[i];
            sum += d * d;
        }
        sum
    }

    pub fn face_neighbor_offsets() -> Vec<Coord7D> {
        let mut offsets = Vec::new();
        for dim in 0..DIM {
            for sign in [-1i32, 1i32] {
                let mut basis = [0i32; DIM];
                basis[dim] = sign;
                offsets.push(Coord7D::new_even(basis));
            }
        }
        offsets
    }

    pub fn shifted(&self, offset: &Coord7D) -> Coord7D {
        let mut new_basis = [0i32; DIM];
        for (i, (a, b)) in self.basis.iter().zip(offset.basis.iter()).enumerate() {
            new_basis[i] = a + b;
        }
        let new_parity = match (self.parity, offset.parity) {
            (Parity::Even, Parity::Even) => Parity::Even,
            (Parity::Even, Parity::Odd) => Parity::Odd,
            (Parity::Odd, Parity::Even) => Parity::Odd,
            (Parity::Odd, Parity::Odd) => Parity::Even,
        };
        Coord7D {
            basis: new_basis,
            parity: new_parity,
        }
    }
}

impl fmt::Display for Coord7D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = self.as_f64();
        write!(
            f,
            "({:.1},{:.1},{:.1} | {:.1},{:.1},{:.1},{:.1})",
            v[0], v[1], v[2], v[3], v[4], v[5], v[6]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_coord_is_integer() {
        let c = Coord7D::new_even([1, 2, 3, 0, 0, 0, 0]);
        let v = c.as_f64();
        assert_eq!(v[0], 1.0);
        assert_eq!(v[6], 0.0);
    }

    #[test]
    fn odd_coord_is_half_integer() {
        let c = Coord7D::new_odd([1, 2, 3, 0, 0, 0, 0]);
        let v = c.as_f64();
        assert_eq!(v[0], 1.5);
        assert_eq!(v[3], 0.5);
    }

    #[test]
    fn physical_dark_split() {
        let c = Coord7D::new_even([1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(c.physical(), [1, 2, 3]);
        assert_eq!(c.dark(), [4, 5, 6, 7]);
    }

    #[test]
    fn distance_between_neighbors() {
        let origin = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let neighbor = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        assert!((origin.distance_sq(&neighbor) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn face_offsets_count() {
        assert_eq!(Coord7D::face_neighbor_offsets().len(), 14);
    }

    #[test]
    fn shifted_parity() {
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_odd([0; 7]);
        let result = a.shifted(&b);
        assert_eq!(result.parity(), Parity::Odd);
    }
}
