use crate::resources::RetainedUiSchedule;

pub fn insert_ui_resources(resources: &mut nightshade::ecs::ResourceMap) {
    resources.insert(crate::resources::RetainedUiRuntime::default());
    resources.insert(crate::resources::ThemeState::default());
    resources.insert(crate::resources::RetainedUiInteraction::default());
    resources.insert(crate::resources::RetainedUiOverlays::default());
    resources.insert(crate::resources::RetainedUiTiming::default());
    resources.insert(crate::resources::RetainedUiDirty::default());
    resources.insert(crate::resources::RetainedUiTextCache::default());
    resources.insert(crate::resources::RetainedUiDragDrop::default());
    resources.insert(crate::resources::RetainedUiAccessibility::default());
    resources.insert(crate::resources::RetainedUiGroups::default());
    resources.insert(crate::resources::RetainedUiFrame::default());
    resources.insert(crate::resources::taffy::UiTaffy::default());
    resources.insert(crate::resources::RetainedUiInputMirror::default());
    resources.insert(crate::resources::TopProgressBar::default());
    resources.insert(crate::resources::IconSet::default());
    resources.insert(crate::resources::DockState::default());
    resources.insert(crate::resources::RetainedUiGamepadNav::default());
    resources.insert(crate::resources::UiViewport::default());
    resources.insert(crate::resources::WindowChrome::default());
    resources.insert(crate::resources::UiImageLayerAllocator::default());
    nightshade_paint::install_paint_resources(resources);
}

pub fn compose(resources: &mut nightshade::ecs::ResourceMap) {
    insert_ui_resources(resources);
    resources.insert(RetainedUiSchedule::default());
}
