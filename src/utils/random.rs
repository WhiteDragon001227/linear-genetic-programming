use std::{cell::UnsafeCell, rc::Rc};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub const SEED_NO: u64 = 42;

type InternalGenerator = Rc<UnsafeCell<ChaCha8Rng>>;

thread_local! {
    static GENERATOR: InternalGenerator = {
        let generator = ChaCha8Rng::seed_from_u64(SEED_NO);
        Rc::new(UnsafeCell::new(generator))
    }
}

pub struct Random {
    rng: InternalGenerator,
}

pub fn generator() -> Random {
    let rng = GENERATOR.with(|t| t.clone());
    Random { rng }
}

impl Default for Random {
    fn default() -> Self {
        self::generator()
    }
}

impl RngCore for Random {
    fn next_u32(&mut self) -> u32 {
        let rng = unsafe { &mut *self.rng.get() };
        rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        let rng = unsafe { &mut *self.rng.get() };
        rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let rng = unsafe { &mut *self.rng.get() };
        rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        let rng = unsafe { &mut *self.rng.get() };
        rng.try_fill_bytes(dest)
    }
}
