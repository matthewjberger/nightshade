use nightshade_platform::KeyCode;
use serde::Serialize;

use nalgebra_glm::Vec2;

#[derive(Clone, Debug)]
pub(super) enum TestAction {
    MoveMouseToEntity(String),
    ClickAtEntity(String),
    SetSliderValue(String, f32),
    SelectDropdownOption(String, usize),
    SelectTab(String, usize),
    TypeChars(Vec<char>),
    PressAndReleaseKey(KeyCode),
    Scroll(Vec2),
    Assert(String, AssertKind),
    Log(String),
}

#[derive(Clone, Debug)]
pub(super) enum AssertKind {
    Visible(String),
    NotVisible(String),
    WidgetValueF32(String, f32, f32),
    WidgetValueBool(String, bool),
    EventFired(EventMatch),
}

#[derive(Clone, Debug)]
pub(super) enum EventMatch {
    ButtonClicked,
    SliderChanged,
    ToggleChanged,
    TextInputChanged,
    CheckboxChanged,
    DropdownChanged,
    RadioChanged,
    TabChanged,
    SelectableLabelClicked,
}

pub(super) struct ScheduledAction {
    pub frame: u64,
    pub action: TestAction,
}

#[derive(Clone, Debug, Serialize)]
pub struct UiTestResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ResultRecord {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub frame: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ActionRecord {
    pub frame: u64,
    pub kind: String,
    pub target: Option<String>,
    pub position: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ReportSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct TestReport {
    pub test_name: String,
    pub passed: bool,
    pub summary: ReportSummary,
    pub results: Vec<ResultRecord>,
    pub actions: Vec<ActionRecord>,
    pub screenshots: Vec<String>,
}

nightshade::ecs::resource_bundle! {
    pub struct TestStage<'a> {
        write {
            input: nightshade_platform::contract::input::Input,
            window: nightshade_platform::contract::window::Window,
            runtime: nightshade_ui::resources::RetainedUiRuntime,
        }
        read {
            hierarchy: nightshade::ecs::HierarchyIndex,
            frame: nightshade_ui::resources::RetainedUiFrame,
            accessibility: nightshade_ui::resources::RetainedUiAccessibility,
        }
    }
}

pub fn test_stage(resources: &mut nightshade::ecs::ResourceMap) -> TestStage<'_> {
    use std::any::TypeId;
    let [input, window, runtime, hierarchy, frame, accessibility] = resources.res_disjoint_mut([
        TypeId::of::<nightshade_platform::contract::input::Input>(),
        TypeId::of::<nightshade_platform::contract::window::Window>(),
        TypeId::of::<nightshade_ui::resources::RetainedUiRuntime>(),
        TypeId::of::<nightshade::ecs::HierarchyIndex>(),
        TypeId::of::<nightshade_ui::resources::RetainedUiFrame>(),
        TypeId::of::<nightshade_ui::resources::RetainedUiAccessibility>(),
    ]);
    TestStage {
        input: input.downcast_mut().expect("Input"),
        window: window.downcast_mut().expect("Window"),
        runtime: runtime.downcast_mut().expect("RetainedUiRuntime"),
        hierarchy: hierarchy.downcast_ref().expect("HierarchyIndex"),
        frame: frame.downcast_ref().expect("RetainedUiFrame"),
        accessibility: accessibility
            .downcast_ref()
            .expect("RetainedUiAccessibility"),
    }
}
