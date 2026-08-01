use nalgebra_glm;
use nalgebra_glm::Vec2;

use crate::components::layout_types::UiLayoutType;
use crate::data::units::UiValue;

use super::UiLayoutNode;

#[derive(Clone, Copy, Debug, Default)]
pub struct ResponsiveOverride {
    pub size: Option<UiValue<Vec2>>,
    pub position_1: Option<UiValue<Vec2>>,
    pub position_2: Option<UiValue<Vec2>>,
    pub visible: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct UiResponsive {
    pub overrides: [ResponsiveOverride; 3],
    pub baseline: ResponsiveOverride,
    pub baseline_captured: bool,
    pub current_applied: Option<UiBreakpoint>,
}

impl ResponsiveOverride {
    pub fn capture_from(node: &UiLayoutNode) -> Self {
        let mut over = ResponsiveOverride {
            visible: Some(node.visible),
            ..Default::default()
        };
        if let Some(layout) = node.base_layout.as_ref() {
            match layout {
                UiLayoutType::Window(window) => {
                    over.size = Some(window.size);
                }
                UiLayoutType::Boundary(boundary) => {
                    over.position_1 = Some(boundary.position_1);
                    over.position_2 = Some(boundary.position_2);
                }
                UiLayoutType::Solid(_) => {}
            }
        }
        over
    }

    pub fn apply_to(&self, node: &mut UiLayoutNode) {
        if let Some(visible) = self.visible {
            node.visible = visible;
        }
        match node.base_layout.as_mut() {
            Some(UiLayoutType::Window(window)) => {
                if let Some(size) = self.size {
                    window.size = size;
                }
            }
            Some(UiLayoutType::Boundary(boundary)) => {
                if let Some(position_1) = self.position_1 {
                    boundary.position_1 = position_1;
                }
                if let Some(position_2) = self.position_2 {
                    boundary.position_2 = position_2;
                }
            }
            _ => {}
        }
    }
}

impl UiBreakpoint {
    pub fn index(self) -> usize {
        match self {
            UiBreakpoint::Compact => 0,
            UiBreakpoint::Medium => 1,
            UiBreakpoint::Wide => 2,
        }
    }
}

impl UiResponsive {
    pub fn override_mut(&mut self, breakpoint: UiBreakpoint) -> &mut ResponsiveOverride {
        &mut self.overrides[breakpoint.index()]
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum UiBreakpoint {
    Compact,
    Medium,
    #[default]
    Wide,
}

impl UiBreakpoint {
    pub fn from_width(width: f32) -> Self {
        if width < 720.0 {
            UiBreakpoint::Compact
        } else if width < 1024.0 {
            UiBreakpoint::Medium
        } else {
            UiBreakpoint::Wide
        }
    }
}
