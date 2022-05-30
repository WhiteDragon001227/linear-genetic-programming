use rand::{Rng, SeedableRng};

pub const SEED_NO: u64 = 42;

pub struct GENERATOR;

impl GENERATOR {
    pub fn get() -> impl Rng {
        rand_chacha::ChaCha8Rng::seed_from_u64(SEED_NO)
    }
}
