# nightshade-waves

A list of waves and the clock that sends them out one at a time. Each `Wave` says what kind arrives, how many of them, how long to wait between two of them, and how long to rest before the next wave starts. The crate announces an arrival rather than spawning anything, so the game decides what a kind looks like and where it comes in.

## Composing it

```rust
use nightshade::app::App;
use nightshade_waves::{Wave, WavePlan, WavesPlugin};

fn build(app: &mut App) {
    app.add_plugin(WavesPlugin);
    app.insert_resource(WavePlan(vec![Wave {
        kind: 0,
        count: 6,
        spacing: 0.35,
        rest: 2.0,
    }]));
}
```

The plugin makes sure a `WavePlan` is there, inserts a fresh `WaveProgress`, and registers `send_the_wave` on `Stage::Update`, where the clock decides and asks. It announces `Arrival` for every one that comes out and `WaveCleared` when a wave has sent its last.

## Using it

```rust
use nightshade_waves::commands::{rewind, start, stop};
use nightshade_waves::queries::{current, finished};
use nightshade_waves::{Arrival, WaveCleared, WavePlan, WaveProgress};

start(&mut progress);

for arrival in arrivals.iter() {
    let which = arrival.kind;
    let place = arrival.index;
}

let running = current(&plan, &progress);
let over = finished(&plan, &progress);
```

Nothing is sent until `start` sets the progress running, and `stop` holds it where it stands while `rewind` puts it back at the first wave. `Arrival` carries the kind of the wave and the index of this one within it, which is enough for a game to place a new body without reading the plan. `WaveCleared` carries the index of the wave that just emptied, and the rest of that wave runs before the next one begins. `current` reads the wave being sent and `finished` reports that the plan has run out.

## Driving it

```
just example waves
```

Space opens and holds the gate through `start` and `stop`, and R winds the run back to the first wave through `rewind`. A body stands in the lane for every `Arrival` the crate sends, tinted by the kind of its wave, and the readout gives the wave, the count sent, the count of `WaveCleared` events and whether the gate is open.
