use rand::{prelude::ThreadRng, thread_rng};

pub const GENERATOR: &mut ThreadRng = &mut thread_rng();
