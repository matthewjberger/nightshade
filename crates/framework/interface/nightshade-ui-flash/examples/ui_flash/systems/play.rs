use crate::resources::Bench;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_ui_flash::Flash;
use nightshade_ui_flash::commands::{clear, flash};
use nightshade_ui_flash::queries::{fading, strength, wash};

pub fn take_the_orders(input: Res<Input>, mut bench: ResMut<Bench>, mut screen: ResMut<Flash>) {
    let asked = if input.keyboard.just_pressed(KeyCode::Digit1) {
        Some((theme::HURT, theme::HURT_FADE, theme::NAMES[0]))
    } else if input.keyboard.just_pressed(KeyCode::Digit2) {
        Some((theme::SHIFT, theme::SHIFT_FADE, theme::NAMES[1]))
    } else if input.keyboard.just_pressed(KeyCode::Digit3) {
        Some((theme::MUZZLE, theme::MUZZLE_FADE, theme::NAMES[2]))
    } else {
        None
    };
    if let Some((color, fade, name)) = asked {
        flash(&mut screen, color, fade);
        bench.fired += 1;
        bench.last = name;
    }
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        clear(&mut screen);
    }
}

pub fn show_the_bench(screen: Res<Flash>, bench: Res<Bench>, mut readout: ResMut<Readout>) {
    let color = wash(&screen);
    readout.notes.clear();
    readout.notes.push(format!(
        "wash {:.2}, {:.2}, {:.2} at {:.2}",
        color.x, color.y, color.z, color.w
    ));
    readout
        .notes
        .push(format!("strongest {:.2}", strength(&screen)));
    readout.notes.push(if fading(&screen) {
        format!("{} washes fading", screen.washes.len())
    } else {
        "the screen is clear".to_string()
    });
    if bench.fired > 0 {
        readout
            .notes
            .push(format!("{} fired, last was {}", bench.fired, bench.last));
    }
}
