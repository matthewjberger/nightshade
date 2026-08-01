use nalgebra_glm::Vec2;
use nightshade_platform::KeyCode;

use nightshade_ui::resources::UiEvent;

use super::types::{
    ActionRecord, AssertKind, EventMatch, ScheduledAction, TestAction, UiTestResult,
};

pub struct UiTestDriver {
    pub(super) actions: Vec<ScheduledAction>,
    pub(super) frame: u64,
    pub(super) frame_cursor: u64,
    pub(super) results: Vec<UiTestResult>,
    pub(super) captured_events: Vec<UiEvent>,
    pub(super) active: bool,
    pub(super) exit_on_complete: bool,
    pub(super) cursor: usize,
    pub(super) pending_release: bool,
    pub(super) pending_right_release: bool,
    pub(super) pending_key_releases: Vec<KeyCode>,
    pub(super) test_name: String,
    pub(super) json_output_path: Option<String>,
    pub(super) action_records: Vec<ActionRecord>,
    pub(super) screenshots: Vec<String>,
    pub(super) result_frames: Vec<u64>,
}

impl UiTestDriver {
    pub fn new(name: &str) -> Self {
        Self {
            actions: Vec::new(),
            frame: 0,
            frame_cursor: 0,
            results: Vec::new(),
            captured_events: Vec::new(),
            active: false,
            exit_on_complete: true,
            cursor: 0,
            pending_release: false,
            pending_right_release: false,
            pending_key_releases: Vec::new(),
            test_name: name.to_string(),
            json_output_path: None,
            action_records: Vec::new(),
            screenshots: Vec::new(),
            result_frames: Vec::new(),
        }
    }

    pub fn set_exit_on_complete(&mut self, exit: bool) {
        self.exit_on_complete = exit;
    }

    pub fn set_json_output_path(&mut self, path: impl Into<String>) {
        self.json_output_path = Some(path.into());
    }

    pub fn start(&mut self) {
        self.active = true;
        self.frame = 0;
        self.cursor = 0;
        self.results.clear();
        self.captured_events.clear();
        self.actions.sort_by_key(|action| action.frame);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|result| result.passed)
    }

    pub(super) fn schedule(&mut self, frame: u64, action: TestAction) {
        self.actions.push(ScheduledAction { frame, action });
    }

    pub fn at_frame(&mut self, frame: u64) -> ActionBuilder<'_> {
        self.frame_cursor = frame;
        ActionBuilder {
            driver: self,
            frame,
        }
    }

    pub fn wait(&mut self, frames: u64) -> ActionBuilder<'_> {
        self.frame_cursor += frames;
        let frame = self.frame_cursor;
        ActionBuilder {
            driver: self,
            frame,
        }
    }

    pub fn then(&mut self) -> ActionBuilder<'_> {
        let frame = self.frame_cursor;
        ActionBuilder {
            driver: self,
            frame,
        }
    }
}

pub struct ActionBuilder<'a> {
    driver: &'a mut UiTestDriver,
    frame: u64,
}

impl ActionBuilder<'_> {
    fn act(self, action: TestAction) -> Self {
        self.driver.schedule(self.frame, action);
        self
    }

    fn assert(self, name: String, kind: AssertKind) -> Self {
        self.act(TestAction::Assert(name, kind))
    }

    pub fn move_mouse_to(self, test_id: &str) -> Self {
        self.act(TestAction::MoveMouseToEntity(test_id.to_string()))
    }

    pub fn click_entity(self, test_id: &str) -> Self {
        self.act(TestAction::ClickAtEntity(test_id.to_string()))
    }

    pub fn set_slider_value(self, test_id: &str, value: f32) -> Self {
        self.act(TestAction::SetSliderValue(test_id.to_string(), value))
    }

    pub fn select_dropdown_option(self, test_id: &str, option_index: usize) -> Self {
        self.act(TestAction::SelectDropdownOption(
            test_id.to_string(),
            option_index,
        ))
    }

    pub fn select_tab(self, test_id: &str, tab_index: usize) -> Self {
        self.act(TestAction::SelectTab(test_id.to_string(), tab_index))
    }

    pub fn type_text(self, text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        self.act(TestAction::TypeChars(chars))
    }

    pub fn scroll(self, delta: Vec2) -> Self {
        self.act(TestAction::Scroll(delta))
    }

    pub fn log(self, message: &str) -> Self {
        self.act(TestAction::Log(message.to_string()))
    }

    pub fn assert_visible(self, test_id: &str) -> Self {
        self.assert(
            format!("{test_id} visible"),
            AssertKind::Visible(test_id.to_string()),
        )
    }

    pub fn assert_not_visible(self, test_id: &str) -> Self {
        self.assert(
            format!("{test_id} not visible"),
            AssertKind::NotVisible(test_id.to_string()),
        )
    }

    pub fn assert_value_f32(self, test_id: &str, expected: f32) -> Self {
        self.assert(
            format!("{test_id} value"),
            AssertKind::WidgetValueF32(test_id.to_string(), expected, 0.01),
        )
    }

    pub fn assert_value_f32_approx(self, test_id: &str, expected: f32, tolerance: f32) -> Self {
        self.assert(
            format!("{test_id} value (approx)"),
            AssertKind::WidgetValueF32(test_id.to_string(), expected, tolerance),
        )
    }

    pub fn assert_value_bool(self, test_id: &str, expected: bool) -> Self {
        self.assert(
            format!("{test_id} value"),
            AssertKind::WidgetValueBool(test_id.to_string(), expected),
        )
    }

    fn assert_event(self, name: &str, event_match: EventMatch) -> Self {
        self.assert(name.to_string(), AssertKind::EventFired(event_match))
    }

    pub fn assert_event_fired_button_clicked(self) -> Self {
        self.assert_event("button clicked event", EventMatch::ButtonClicked)
    }

    pub fn assert_event_fired_slider_changed(self) -> Self {
        self.assert_event("slider changed event", EventMatch::SliderChanged)
    }

    pub fn assert_event_fired_toggle_changed(self) -> Self {
        self.assert_event("toggle changed event", EventMatch::ToggleChanged)
    }

    pub fn assert_event_fired_text_changed(self) -> Self {
        self.assert_event("text input changed event", EventMatch::TextInputChanged)
    }

    pub fn assert_event_fired_checkbox_changed(self) -> Self {
        self.assert_event("checkbox changed event", EventMatch::CheckboxChanged)
    }

    pub fn assert_event_fired_dropdown_changed(self) -> Self {
        self.assert_event("dropdown changed event", EventMatch::DropdownChanged)
    }

    pub fn assert_event_fired_radio_changed(self) -> Self {
        self.assert_event("radio changed event", EventMatch::RadioChanged)
    }

    pub fn assert_event_fired_tab_changed(self) -> Self {
        self.assert_event("tab changed event", EventMatch::TabChanged)
    }

    pub fn assert_event_fired_selectable_label_clicked(self) -> Self {
        self.assert_event(
            "selectable label clicked event",
            EventMatch::SelectableLabelClicked,
        )
    }
}

pub fn setup_test_scene(stage: &mut crate::types::TestStage<'_>) {
    let runtime = &mut *stage.runtime;
    runtime.visible = true;
    runtime.background_color = Some(nalgebra_glm::Vec4::new(0.05, 0.05, 0.08, 1.0));
}

pub fn create_test_root(
    tree: &mut nightshade_ui::commands::builder::UiTreeBuilder<'_, '_>,
) -> nightshade::ecs::Entity {
    tree.add_node()
        .boundary(
            nightshade_ui::data::units::Ab(Vec2::new(20.0, 20.0)),
            nightshade_ui::data::units::Ab(Vec2::new(-20.0, -20.0))
                + nightshade_ui::data::units::Rl(Vec2::new(100.0, 100.0)),
        )
        .flow(
            nightshade_ui::components::layout_types::FlowDirection::Vertical,
            8.0,
            8.0,
        )
        .entity()
}
