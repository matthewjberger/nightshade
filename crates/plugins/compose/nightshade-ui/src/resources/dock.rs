#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ReservedAreas {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PanelDockConfig {
    pub id: String,
    pub title: String,
    pub kind: crate::components::UiPanelKind,
    pub focus_order: i32,
    pub size: f32,
    pub visible: bool,
    pub collapsed: bool,
    pub floating_position: Option<[f32; 2]>,
    pub floating_size: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct WindowDockLayout {
    pub reserved_areas: ReservedAreas,
    pub panels: Vec<PanelDockConfig>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DockState {
    pub primary: WindowDockLayout,
}
