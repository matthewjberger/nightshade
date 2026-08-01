use nightshade::prelude::Entity;
use nightshade_cli::clap;

#[derive(clap::Args, Default)]
pub struct SponzaArguments {
    #[arg(long, help = "Open on foot rather than in the orbit")]
    pub walk: bool,
    #[arg(long, help = "Open with the panel of knobs shown")]
    pub panel: bool,
}

#[derive(Default)]
pub struct Walk {
    pub orbit: Option<Entity>,
    pub body: Option<Entity>,
    pub eyes: Option<Entity>,
    pub walking: bool,
    pub wanted: Option<bool>,
}

#[derive(Default)]
pub struct Handles {
    pub sun: Option<Entity>,
    pub lanterns: [Option<Entity>; 3],
    pub model: Option<nightshade_load::LoadHandle>,
}

#[derive(Default, Clone, Copy)]
pub struct Panel {
    pub docked: Entity,
    pub shown: bool,
    pub wanted: bool,
    pub rate_in_title_on: bool,
    pub title_rate: Option<i32>,
    pub label_rate: i32,
    pub rate: Entity,
    pub rate_in_title: Entity,
    pub render_scale: Entity,
    pub traced_sun: Entity,
    pub traced_reflections: Entity,
    pub traced_occlusion: Entity,
    pub occlusion: Entity,
    pub occlusion_strength: Entity,
    pub reflections: Entity,
    pub reflection_strength: Entity,
    pub bounced: Entity,
    pub bounced_strength: Entity,
    pub lit_air: Entity,
    pub air_density: Entity,
    pub air_forward: Entity,
    pub air_falloff: Entity,
    pub air_reach: Entity,
    pub sun_intensity: Entity,
    pub sun_yaw: Entity,
    pub sun_pitch: Entity,
    pub lantern_intensity: Entity,
    pub lantern_range: Entity,
    pub bloom: Entity,
    pub bloom_strength: Entity,
    pub anti_aliasing: Entity,
    pub sharpen: Entity,
    pub focus: Entity,
    pub focus_blur: Entity,
    pub exposure_follows: Entity,
    pub exposure_target: Entity,
}
