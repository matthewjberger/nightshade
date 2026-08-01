use crate::data::Picture;
use nightshade::ecs::Entity;
use nightshade_load::Loader;

pub type Pictures = Loader<Picture>;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Loading,
    Playing,
}

#[derive(Default)]
pub struct Progress {
    pub models_landed: usize,
    pub pictures_landed: usize,
    pub sounds_landed: usize,
    pub failed: usize,
    pub last_count: String,
    pub last_reading: String,
}

#[derive(Default, Clone, Copy)]
pub struct Handles {
    pub card: Entity,
    pub bar_fill: Entity,
    pub count: Entity,
    pub name: Entity,
    pub play_root: Entity,
    pub reload: Entity,
    pub picture_tree: Entity,
    pub camera: Entity,
    pub sun: Entity,
    pub played: bool,
}
