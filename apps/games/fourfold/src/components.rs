use nightshade::prelude::Vec3;

#[derive(Clone, Copy, Debug, Default)]
pub struct Extent(pub Vec3);

#[derive(Clone, Copy, Debug, Default)]
pub struct Dial(pub usize);

#[derive(Clone, Copy, Debug, Default)]
pub struct Pip(pub usize);

#[derive(Clone, Copy, Debug, Default)]
pub struct Cell(pub [i32; 2]);
