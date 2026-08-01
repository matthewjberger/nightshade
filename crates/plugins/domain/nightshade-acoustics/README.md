# nightshade-acoustics

nightshade-acoustics traces sound through a voxel grid of materials and reports what a listener hears. A program lays out the grid, marks entities as emitters and one entity as the listener, then reads back the occlusion, permeation, reverb and echo readings each frame. The crate produces readings and plays nothing, so `nightshade-audio` is what turns them into sound.

## Composing it

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(nightshade_audio::AudioPlugin)
    .add_plugin(nightshade_acoustics::AcousticsPlugin)
    .run()
```

`AcousticsPlugin` inserts `AcousticsFrame`, `AcousticsEcho`, `AcousticsGrid`, `AcousticsHeard`, `AcousticsRays`, `AcousticsRoom` and `AcousticsSettings`, then registers `gather_acoustics_system` on `Stage::Render`.

## Using it

```rust
use nightshade_acoustics::commands::{acoustics_grid_fill, acoustics_grid_lay_out};
use nightshade_acoustics::contract::{
    AcousticEmitter, AcousticListener, AcousticsGrid, EmitterVoice, STONE,
};

pub fn build_the_room(
    mut grid: ResMut<AcousticsGrid>,
    mut commands: Commands,
    demo: Res<AcousticsDemo>,
) {
    acoustics_grid_lay_out(&mut grid, Vec3::new(-32.0, -8.0, -32.0), 1.0, 64);
    acoustics_grid_fill(
        &mut grid,
        Vec3::new(-8.0, 0.0, -8.0),
        Vec3::new(8.0, 4.0, 8.0),
        STONE,
    );
    commands.set(
        demo.source,
        AcousticEmitter::new(40.0)
            .with_voice(EmitterVoice::Tone)
            .with_own_reverb(),
    );
    commands.set(demo.head, AcousticListener);
}
```

`acoustics_grid_lay_out` sets the origin, the cell size and the side length, and `acoustics_grid_fill` writes a material code into a box of cells. `AcousticEmitter` carries the reach, the voice and the ray budgets, and its builder methods set each of them. `AcousticListener` on an entity is where the readings are taken from. A system then reads `AcousticsFrame` for the emitter places, `AcousticsRoom` for the room and zone readings, and `acoustics_path_of` in `queries` for one emitter's `SoundPath`.

## Taking it from another crate

The crate splits at a `contract` feature, so a crate that only names the types writes `nightshade-acoustics = { path = "../nightshade-acoustics", default-features = false, features = ["contract"] }`. The contract carries the two components, the seven resources, the material table and the grid constants, with none of the tracing behind them.
