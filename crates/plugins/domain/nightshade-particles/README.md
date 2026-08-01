# nightshade-particles

`nightshade-particles` simulates emitters on the CPU and gathers them into the frame the backends draw. An entity that carries a `ParticleEmitter` spawns particles by the rules the component states, ages them against gravity, drag and turbulence, and fades them along a color gradient. The crate holds the emitter shapes, the presets a game reaches for, and the texture uploads that give particles a picture.

## Composing it

```rust
nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled("Fireworks"))
    .add_plugin(ParticlePlugin)
    .add_plugin(FireworksPlugin)
    .run()
```

`ParticlePlugin` inserts `ParticleTextures` and `ParticleFrame`, steps the emitters on `Stage::FrameUpdate`, and gathers them into the frame on `Stage::Render`.

## Using it

```rust
pub fn sparkle(commands: &mut Commands, position: Vec3) {
    let entity = commands.spawn(());
    let mut emitter = ParticleEmitter::firework_glitter(position, 60);
    emitter.initial_velocity_min = 0.6;
    emitter.initial_velocity_max = 1.8;
    emitter.size_start = 0.05;
    emitter.size_end = 0.0;
    emitter.gravity = Vec3::new(0.0, -3.0, 0.0);
    commands.set(entity, emitter);
    commands.set(entity, Lifetime { remaining: 3.0 });
}
```

A `ParticleEmitter` is an ordinary component, so setting it on an entity is the whole of starting an effect. The presets on the type, among them `firework_glitter`, `firework_explosion` and `smoke`, give a starting point a game then edits field by field. `one_shot` with a `burst_count` fires once and stops, while a `spawn_rate` keeps a stream running until `enabled` goes false. `EmitterShape` says where particles appear, `EmitterType` picks the look, and `queue_particle_texture` pushes a `ParticleTextureUpload` into `ParticleTextures` so an emitter's `texture_index` names a picture.

## Taking it from another crate

The crate splits at a `contract` feature, which is how both backends read the frame without linking the simulation.

```toml
nightshade-particles = { path = "../nightshade-particles", default-features = false, features = ["contract"] }
```

The contract carries the emitter component, the shapes and gradients, the presets, and the `ParticleFrame` and `ParticleTextures` resources.
