# nightshade-audio

nightshade-audio plays sound. An `AudioSource` on an entity is a clip with a volume, a bus and optional spatialization, and the crate loads the bytes, builds the mixer tracks and keeps the playing handles in step with the world each frame. It also carries the buses, the voice ducking, the live streams and the capture devices, and it reads the acoustics contract so a traced room can shape what is heard.

## Composing it

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(nightshade_audio::AudioPlugin)
    .run()
```

`AudioPlugin` inserts `AudioEngine` and `Sounds`, and registers the startup system that opens the device. The frame systems run on `Stage::FrameStart`: the request drain, the sound collection, the bus build, the listener sync, the source update, the pause sync and the acoustics drive.

## Using it

```rust
use nightshade_audio::contract::{AudioBus, AudioListener, AudioSource, PendingSound};

let room = commands.spawn((AudioSource {
    audio_ref: Some("room".to_string()),
    volume: 0.4,
    looping: true,
    playing: true,
    bus: AudioBus::Ambient,
    ..Default::default()
},));
commands.set(head, AudioListener);

commands.spawn((PendingSound {
    clip: "footstep".to_string(),
    volume: 1.0,
    ..Default::default()
},));
```

`AudioSource` names the clip in `audio_ref` and carries the volume, the looping flag, the bus and the distance falloff, and `with_looping`, `with_spatial`, `with_bus` and `with_distance` build one without naming every field. `AudioListener` on an entity is where the spatial mix is taken from. `PendingSound` is the ask a capability makes without depending on the engine, and the request drain plays it and takes the component away. A system that wants the engine itself takes `ResMut<AudioEngine>` and calls `set_audio_bus_volume`, `audio_engine_stop_sound` or `audio_engine_fade_volume` from `commands`.

## Taking it from another crate

The crate splits at a `contract` feature, so a crate that only names the types writes `nightshade-audio = { path = "../nightshade-audio", default-features = false, features = ["contract"] }`. The contract carries `AudioSource`, `AudioListener`, `AudioBus`, `AudioTrack` and `PendingSound`, and pulls in no mixer and no device.
