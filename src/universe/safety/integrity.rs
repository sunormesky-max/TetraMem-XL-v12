use sha2::{Digest, Sha256};

pub struct IntegrityHasher {
    inner: Sha256,
}

impl IntegrityHasher {
    pub fn new() -> Self {
        Self {
            inner: Sha256::new(),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    pub fn finalize(self) -> String {
        let result = self.inner.finalize();
        format!("{:x}", result)
    }
}

impl Default for IntegrityHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_deterministic() {
        let mut h1 = IntegrityHasher::new();
        h1.update(b"hello");
        let r1 = h1.finalize();

        let mut h2 = IntegrityHasher::new();
        h2.update(b"hello");
        let r2 = h2.finalize();

        assert_eq!(r1, r2);
        assert_eq!(r1.len(), 64);
    }

    #[test]
    fn hash_differs_for_different_input() {
        let mut h1 = IntegrityHasher::new();
        h1.update(b"hello");
        let r1 = h1.finalize();

        let mut h2 = IntegrityHasher::new();
        h2.update(b"world");
        let r2 = h2.finalize();

        assert_ne!(r1, r2);
    }
}
