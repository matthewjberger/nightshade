use nightshade::prelude::{Vec3, vec4};
use nightshade_ui_story::StoryStyle;

pub const TITLE: &str = "Ui Story";

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 8.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.0;

pub fn style() -> StoryStyle {
    StoryStyle {
        width: 820.0,
        lines: 13,
        columns: 68,
        links: 5,
        line_size: 17.0,
        background: vec4(0.05, 0.05, 0.08, 0.95),
        border: vec4(0.44, 0.48, 0.66, 0.85),
        ..StoryStyle::default()
    }
}

pub const SOURCE: &str = "
story The Short Way Home
opening You are late, and the gate is shut.
start yard

value opened 0

room yard
  title The Yard
  look Rain on the flagstones. The gate stands shut in the wall.
  way through garden when opened same 1 barred The gate is shut fast.

room garden
  title The Wet Garden
  look Beans on their poles, and the kitchen light on at the far end.
  way back yard

thing key
  words key iron
  look A cold iron key, bright where a thumb has worn it.
  at yard
  portable
  taking You take the key.

thing gate
  words gate lock
  look A gate of black iron, with a lock at the height of your hand.
  at yard

verb open gate
  when holds key
  say The key turns, and the gate gives.
  set opened set 1
  barred The lock will not turn under your fingers alone.
";
