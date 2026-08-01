use crate::commands::{effects, launch};
use crate::resources::{Launcher, Toggles};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn watch_the_keys(
    mut launcher: ResMut<Launcher>,
    mut toggles: ResMut<Toggles>,
    reader: EventReader<AppEvent>,
    mut commands: Commands,
) {
    for event in reader.iter() {
        let AppEvent::Keyboard {
            key,
            state: KeyState::Pressed,
        } = event
        else {
            continue;
        };
        match key {
            KeyCode::Space => launch::launch_firework(&mut commands),
            KeyCode::Digit1 => launch::launch_salvo(&mut commands, 3),
            KeyCode::Digit2 => launch::launch_salvo(&mut commands, 5),
            KeyCode::Digit3 => launch::launch_salvo(&mut commands, 10),
            KeyCode::Digit4 => launch::launch_finale(&mut commands),
            KeyCode::Digit5 => launcher.salvo = !launcher.salvo,
            KeyCode::KeyA => launcher.auto = !launcher.auto,
            KeyCode::KeyF => {
                toggles.fire = !toggles.fire;
                if toggles.fire {
                    effects::spawn_fire_effect(&mut commands, Vec3::new(-35.0, 0.0, 0.0));
                    effects::spawn_fire_effect(&mut commands, Vec3::new(35.0, 0.0, 0.0));
                }
            }
            KeyCode::KeyS => {
                toggles.smoke = !toggles.smoke;
                if toggles.smoke {
                    effects::spawn_smoke_effect(&mut commands, Vec3::new(-30.0, 3.0, 5.0));
                    effects::spawn_smoke_effect(&mut commands, Vec3::new(30.0, 3.0, 5.0));
                }
            }
            _ => {}
        }
    }
}
