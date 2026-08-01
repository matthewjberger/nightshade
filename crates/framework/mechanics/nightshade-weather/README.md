# nightshade-weather

The sky a game runs under and how long it stays that way. The crate holds one of four skies, counts down the spell it is in, and rolls a new one when the time runs out, with the chance of rain being the number a game picks. Wetness rises while it rains and falls back while it does not, so the ground can stay dark for a while after the sky clears.

## Composing it

```rust
use nightshade::app::App;
use nightshade_weather::{WeatherPlugin, WeatherSettings};

fn build(app: &mut App) {
    app.add_plugin(WeatherPlugin {
        settings: WeatherSettings {
            shortest: 30.0,
            longest: 180.0,
            chance_of_rain: 0.35,
            dry_rate: 0.08,
        },
    });
}
```

The plugin inserts `WeatherSettings` and a clear `Weather`, makes sure a `Roll` is there to be read, and registers `turn_the_weather` on `Stage::Update`, the one writer of the sky and the wetness. Nothing is announced, because the sky is state a game reads whenever it draws.

## Using it

```rust
use nightshade_weather::commands::roll_the_sky;
use nightshade_weather::queries::{cover, raining};
use nightshade_weather::{Sky, Weather};

let wet = raining(&weather);
let thick = cover(&weather);
let left = weather.remaining;

match weather.sky {
    Sky::Clear => {}
    Sky::Cloudy => {}
    Sky::Rain => {}
    Sky::Storm => {}
}

roll_the_sky(&mut weather, &settings, &mut roll);
```

`raining` is true under rain and under a storm, and `cover` gives a share from zero under a clear sky to one under a storm, which is the number to drive a cloud layer or a light from. `remaining` is the seconds left in this spell and `wetness` is the ground's soak, both moved by the frame system. `roll_the_sky` picks the next sky and the length of its spell, which a game calls itself to break the weather early.

## Driving it

```
just example weather
```

Press 1 for a clear sky, 2 for cloudy, 3 for rain and 4 for a storm. Space rolls a new sky the way the frame system does when a spell runs out. The sky color, the fog and the height of the cloud deck follow the cover, and the readout gives the sky, the time left in it and the wetness.
