pub fn threads() -> usize {
    #[cfg(not(target_family = "wasm"))]
    {
        nightshade::ecs::rayon::current_num_threads()
    }

    #[cfg(target_family = "wasm")]
    1
}

pub fn fill_by_index<T, F>(slots: &mut [T], fill: F)
where
    T: Send,
    F: Fn(usize) -> T + Sync + Send,
{
    #[cfg(not(target_family = "wasm"))]
    {
        const CHUNK: usize = 4096;

        use nightshade::ecs::rayon::prelude::*;

        slots
            .par_chunks_mut(CHUNK)
            .enumerate()
            .for_each(|(chunk, slice)| {
                let base = chunk * CHUNK;
                for (offset, slot) in slice.iter_mut().enumerate() {
                    *slot = fill(base + offset);
                }
            });
    }

    #[cfg(target_family = "wasm")]
    for (index, slot) in slots.iter_mut().enumerate() {
        *slot = fill(index);
    }
}

pub fn fill_runs<T, F>(slots: &mut [T], fill: F)
where
    T: Send,
    F: Fn(usize, &mut [T]) + Sync + Send,
{
    #[cfg(not(target_family = "wasm"))]
    {
        const CHUNK: usize = 4096;

        use nightshade::ecs::rayon::prelude::*;

        slots
            .par_chunks_mut(CHUNK)
            .enumerate()
            .for_each(|(chunk, run)| fill(chunk * CHUNK, run));
    }

    #[cfg(target_family = "wasm")]
    fill(0, slots);
}

pub fn sort(keys: &mut [u64]) {
    #[cfg(not(target_family = "wasm"))]
    {
        use nightshade::ecs::rayon::prelude::*;

        keys.par_sort_unstable();
    }

    #[cfg(target_family = "wasm")]
    keys.sort_unstable();
}
