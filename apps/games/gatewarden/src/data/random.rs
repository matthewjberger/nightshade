use fastrand::Rng;

pub fn random_below(seed: &mut u64, bound: usize) -> usize {
    if bound == 0 {
        return 0;
    }
    let mut rng = Rng::with_seed(*seed);
    let pick = rng.usize(..bound);
    *seed = rng.u64(..);
    pick
}
