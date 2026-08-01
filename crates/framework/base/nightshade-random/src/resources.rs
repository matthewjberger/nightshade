pub struct Random(pub fastrand::Rng);

impl Random {
    pub fn seeded(seed: u64) -> Self {
        Self(fastrand::Rng::with_seed(seed))
    }
}

impl Default for Random {
    fn default() -> Self {
        Self(fastrand::Rng::with_seed(0))
    }
}

impl std::ops::Deref for Random {
    type Target = fastrand::Rng;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Random {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
