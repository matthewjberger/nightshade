use nalgebra_glm::Vec3;

pub const TITLE: &str = "Story";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.13, 0.13, 0.16, 1.0];
pub const ROOM_TINT: [f32; 4] = [0.22, 0.24, 0.30, 1.0];
pub const HERE_TINT: [f32; 4] = [0.96, 0.84, 0.46, 1.0];
pub const GONE_TINT: [f32; 4] = [0.16, 0.16, 0.18, 1.0];
pub const STANDING_TINT: [f32; 4] = [0.46, 0.72, 0.92, 1.0];
pub const HELD_TINT: [f32; 4] = [0.92, 0.52, 0.48, 1.0];

pub const ROOM_SPAN: f32 = 3.0;
pub const THING_SPAN: f32 = 2.2;
pub const ROOM_LIFT: f32 = 2.6;
pub const THING_LIFT: f32 = 0.6;
pub const HERE_RISE: f32 = 1.6;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.6, 0.0);
pub const CAMERA_BACK: f32 = 18.0;
pub const CAMERA_YAW: f32 = 0.4;
pub const CAMERA_PITCH: f32 = 0.42;

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
