use crate::data::Socket;
use nalgebra_glm::{Vec2, Vec4};

#[derive(Default, Clone, Copy, Debug)]
pub struct NodePlace(pub Vec2);

#[derive(Default, Clone, Debug)]
pub struct NodeTitle(pub String);

#[derive(Default, Clone, Copy, Debug)]
pub struct NodeInlets(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct NodeOutlets(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct NodeTint(pub Vec4);

#[derive(Default, Clone, Debug)]
pub struct NodeInletNames(pub Vec<String>);

#[derive(Default, Clone, Debug)]
pub struct NodeOutletNames(pub Vec<String>);

#[derive(Default, Clone, Debug)]
pub struct NodeNote(pub String);

#[derive(Default, Clone, Copy, Debug)]
pub struct NodeField(pub bool);

#[derive(Default, Clone, Copy, Debug)]
pub struct WireFrom(pub Socket);

#[derive(Default, Clone, Copy, Debug)]
pub struct WireTo(pub Socket);
