# nightshade-worker

A crew of threads over a job trait. A caller declares a job type with its own key, its own result and its own per worker tools, hires a crew, submits jobs with a rank, and takes finished results back a bounded number at a time. The crate takes no dependency at all, not even on the engine, so it knows nothing about what the jobs mean.

## Composing it

The crate carries no plugin. A program hires a crew in its own plugin and inserts it as a resource.

```rust
app.insert_resource(workers_hire::<SculptJob>(hands));
```

`workers_hire` takes a cap on the threads and hands back a `Workers<J>`, which is an ordinary resource a system reads as `ResMut<Workers<SculptJob>>`. Half the machine's processors are the budget across all crews, and `workers_free` reports how many hands are still unclaimed.

## Using it

```rust
impl Job for SculptJob {
    type Key = usize;
    type Done = SculptWork;
    type Tools = Chisel;

    fn key(&self) -> Self::Key {
        self.slot
    }

    fn run(self, tools: &mut Self::Tools) -> Self::Done {
        SculptWork {
            slot: self.slot,
            mesh: sculpt(self.seed, self.slot, tools),
            era: self.era,
        }
    }
}

if workers_submit(&mut crew, job, slot as i32) {
    gallery.stage[slot] = Stage::Asked;
}
```

`run` happens on a worker thread and receives the `Tools`, a `Default` scratch value each thread builds once and keeps. `workers_submit` returns false when a job with the same key is already in flight, so a system can ask every frame without piling duplicates up, and the rank orders the queue with the lowest rank drawn first. `workers_take` drains up to a limit of finished results on the calling thread, `workers_cancel` drops a queued job by key, and `workers_clear` empties the queue and starts a new era so results from the old one are discarded. `workers_in_flight`, `workers_queued`, `workers_waiting`, `workers_holds` and `workers_hands` report the state of the crew. On the web there are no threads, so the jobs run inside `workers_take`, up to the limit it was given.

## Taking it from another crate

The crate does not split at a `contract` feature. It has no dependencies and no ECS in it, so a caller takes the whole crate and names its own job type.
