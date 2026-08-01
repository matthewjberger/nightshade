use nightshade::ecs::Storage;

use crate::resources::DemoScene;

pub type DemoMount = fn(&mut Storage, &mut DemoScene<'_>);

pub type DemoStep = fn(&mut Storage, &mut DemoScene<'_>, f32);

#[derive(Clone, Copy, Debug)]
pub enum DemoControl {
    Button {
        label: &'static str,
        action: &'static str,
    },
    Slider {
        label: &'static str,
        action: &'static str,
        lowest: f32,
        highest: f32,
        start: f32,
    },
    Toggle {
        label: &'static str,
        action: &'static str,
        start: bool,
    },
    Choice {
        label: &'static str,
        action: &'static str,
        options: &'static [&'static str],
        start: usize,
    },
}

pub struct DemoEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    pub mount: DemoMount,
    pub step: DemoStep,
    pub controls: &'static [DemoControl],
}
