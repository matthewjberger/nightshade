static VERDICT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

pub fn exit_code() -> i32 {
    VERDICT.load(std::sync::atomic::Ordering::Relaxed)
}

use nalgebra_glm::Vec2;
use nightshade_platform::KeyCode;

use nightshade_platform::contract::input::MouseState;
use nightshade_ui::resources::UiEvent;

use super::types::{
    ActionRecord, AssertKind, EventMatch, ReportSummary, ResultRecord, ScheduledAction, TestAction,
    TestReport, UiTestResult,
};
use crate::driver::UiTestDriver;

impl UiTestDriver {
    fn clear_frame_input(&self, stage: &mut crate::types::TestStage<'_>) {
        stage.input.mouse.state = MouseState::empty();
        stage.input.mouse.position_delta = Vec2::zeros();
        stage.input.mouse.wheel_delta = Vec2::zeros();
        stage.input.keyboard.just_pressed_keys.clear();
        stage.input.keyboard.just_released_keys.clear();
        stage.input.keyboard.frame_keys.clear();
        stage.input.keyboard.frame_chars.clear();
    }

    pub fn step(
        &mut self,
        storage: &mut nightshade::ecs::Storage,
        stage: &mut crate::types::TestStage<'_>,
    ) {
        if !self.active {
            return;
        }

        self.captured_events
            .extend(stage.frame.events.iter().cloned());

        self.clear_frame_input(stage);

        if self.pending_release {
            self.pending_release = false;
            stage
                .input
                .mouse
                .state
                .insert(MouseState::LEFT_JUST_RELEASED);
        }

        if self.pending_right_release {
            self.pending_right_release = false;
            stage
                .input
                .mouse
                .state
                .insert(MouseState::RIGHT_JUST_RELEASED);
        }

        if !self.pending_key_releases.is_empty() {
            let keys = std::mem::take(&mut self.pending_key_releases);
            for key in &keys {
                stage
                    .input
                    .keyboard
                    .keystates
                    .insert(*key, nightshade_platform::ElementState::Released);
                stage.input.keyboard.just_released_keys.insert(*key);
                stage.input.keyboard.frame_keys.push((*key, false));
            }
        }

        self.process_frame_actions(storage, stage);
        self.frame += 1;

        if self.cursor >= self.actions.len()
            && !self.pending_release
            && !self.pending_right_release
            && self.pending_key_releases.is_empty()
        {
            self.active = false;
            self.print_results();
            VERDICT.store(
                if self.all_passed() { 0 } else { 1 },
                std::sync::atomic::Ordering::Relaxed,
            );
            if self.exit_on_complete {
                stage.window.should_exit = true;
            }
        }
    }

    fn process_frame_actions(
        &mut self,
        storage: &mut nightshade::ecs::Storage,
        stage: &mut crate::types::TestStage<'_>,
    ) {
        while self.cursor < self.actions.len() && self.actions[self.cursor].frame == self.frame {
            let action = self.actions[self.cursor].action.clone();
            self.execute_action(storage, stage, action);
            self.cursor += 1;
        }
    }

    pub(super) fn resolve_entity_center(
        &self,
        storage: &nightshade::ecs::Storage,
        stage: &crate::types::TestStage<'_>,
        test_id: &str,
    ) -> Option<Vec2> {
        let entity = *stage.accessibility.test_id_map.get(test_id)?;
        let node = storage.get::<nightshade_ui::components::UiLayoutNode>(entity)?;
        Some(node.computed_rect.center())
    }

    fn resolve_raw_entity_center(
        &self,
        storage: &nightshade::ecs::Storage,
        entity: nightshade::ecs::Entity,
    ) -> Option<Vec2> {
        let node = storage.get::<nightshade_ui::components::UiLayoutNode>(entity)?;
        Some(node.computed_rect.center())
    }

    fn inject_click_at(&mut self, stage: &mut crate::types::TestStage<'_>, position: Vec2) {
        let delta = position - stage.input.mouse.position;
        stage.input.mouse.position = position;
        stage.input.mouse.position_delta = delta;
        stage
            .input
            .mouse
            .state
            .insert(MouseState::MOVED | MouseState::LEFT_JUST_PRESSED | MouseState::LEFT_CLICKED);
        self.pending_release = true;
    }

    fn build_parent_map(
        stage: &crate::types::TestStage<'_>,
    ) -> std::collections::HashMap<nightshade::ecs::Entity, nightshade::ecs::Entity> {
        let mut map = std::collections::HashMap::new();
        for (parent, children) in stage.hierarchy.children.iter() {
            for &child in children {
                map.insert(child, *parent);
            }
        }
        map
    }

    fn scroll_into_view(
        &mut self,
        storage: &mut nightshade::ecs::Storage,
        stage: &mut crate::types::TestStage<'_>,
        test_id: &str,
    ) -> bool {
        let entity = match stage.accessibility.test_id_map.get(test_id).copied() {
            Some(e) => e,
            None => return false,
        };

        let target_center = match storage.get::<nightshade_ui::components::UiLayoutNode>(entity) {
            Some(node) => node.computed_rect.center(),
            None => return false,
        };

        let parents = Self::build_parent_map(stage);
        let mut current = entity;
        let mut outermost_scroll: Option<nightshade::ecs::Entity> = None;

        while let Some(&parent) = parents.get(&current) {
            if storage
                .get::<nightshade_ui::components::UiScrollAreaData>(parent)
                .is_some()
            {
                outermost_scroll = Some(parent);
            }
            current = parent;
        }

        let scroll_entity = match outermost_scroll {
            Some(e) => e,
            None => return false,
        };

        let scroll_rect =
            match storage.get::<nightshade_ui::components::UiLayoutNode>(scroll_entity) {
                Some(node) => node.computed_rect,
                None => return false,
            };

        if scroll_rect.contains(target_center) {
            return false;
        }

        let dpi_scale = stage.window.cached_scale_factor.max(1.0);
        let (current_offset, content_entity) =
            match storage.get::<nightshade_ui::components::UiScrollAreaData>(scroll_entity) {
                Some(data) => (data.scroll_offset, data.content_entity),
                None => return false,
            };

        let scroll_center_y = scroll_rect.center().y;
        let delta_pixels = target_center.y - scroll_center_y;
        let delta_units = delta_pixels / dpi_scale;
        let new_offset = (current_offset + delta_units).max(0.0);

        if (new_offset - current_offset).abs() < 0.5 {
            return false;
        }

        if let Some(data_mut) =
            storage.get_mut::<nightshade_ui::components::UiScrollAreaData>(scroll_entity)
        {
            data_mut.scroll_offset = new_offset;
        }
        if let Some(content_node) =
            storage.get_mut::<nightshade_ui::components::UiLayoutNode>(content_entity)
        {
            content_node.scroll_offset = Vec2::new(0.0, new_offset);
        }

        true
    }

    fn execute_action(
        &mut self,
        storage: &mut nightshade::ecs::Storage,
        stage: &mut crate::types::TestStage<'_>,
        action: TestAction,
    ) {
        self.record_action(&action, storage, stage);
        match action {
            TestAction::MoveMouseToEntity(test_id) => {
                if let Some(center) = self.resolve_entity_center(storage, stage, &test_id) {
                    let delta = center - stage.input.mouse.position;
                    stage.input.mouse.position = center;
                    stage.input.mouse.position_delta = delta;
                    stage.input.mouse.state.insert(MouseState::MOVED);
                } else {
                    eprintln!("[UI_TEST] WARNING: entity '{test_id}' not found for move_mouse_to");
                }
            }
            TestAction::ClickAtEntity(test_id) => {
                if self.scroll_into_view(storage, stage, &test_id) {
                    self.actions.push(ScheduledAction {
                        frame: self.frame + 1,
                        action: TestAction::ClickAtEntity(test_id),
                    });
                    self.actions.sort_by_key(|action| action.frame);
                } else if let Some(center) = self.resolve_entity_center(storage, stage, &test_id) {
                    self.inject_click_at(stage, center);
                } else {
                    eprintln!("[UI_TEST] WARNING: entity '{test_id}' not found for click_entity");
                }
            }
            TestAction::SetSliderValue(test_id, target_value) => {
                if self.scroll_into_view(storage, stage, &test_id) {
                    self.actions.push(ScheduledAction {
                        frame: self.frame + 1,
                        action: TestAction::SetSliderValue(test_id, target_value),
                    });
                    self.actions.sort_by_key(|action| action.frame);
                    return;
                }
                let entity = stage.accessibility.test_id_map.get(&test_id).copied();
                if let Some(entity) = entity {
                    let slider_info = if let Some(data) =
                        storage.get::<nightshade_ui::components::UiSliderData>(entity)
                    {
                        let rect = storage
                            .get::<nightshade_ui::components::UiLayoutNode>(entity)
                            .map(|node| node.computed_rect);
                        rect.map(|rect| {
                            let range = data.max - data.min;
                            let normalized = if data.logarithmic && data.min > 0.0 && range > 0.0 {
                                ((target_value / data.min).ln() / (data.max / data.min).ln())
                                    .clamp(0.0, 1.0)
                            } else if range.abs() > f32::EPSILON {
                                ((target_value - data.min) / range).clamp(0.0, 1.0)
                            } else {
                                0.0
                            };
                            let target_x = rect.min.x + normalized * rect.width();
                            let target_y = rect.center().y;
                            Vec2::new(target_x, target_y)
                        })
                    } else {
                        None
                    };
                    if let Some(position) = slider_info {
                        self.inject_click_at(stage, position);
                    } else {
                        eprintln!(
                            "[UI_TEST] WARNING: '{test_id}' is not a slider or has no layout"
                        );
                    }
                } else {
                    eprintln!(
                        "[UI_TEST] WARNING: entity '{test_id}' not found for set_slider_value"
                    );
                }
            }
            TestAction::SelectDropdownOption(test_id, option_index) => {
                if self.scroll_into_view(storage, stage, &test_id) {
                    self.actions.push(ScheduledAction {
                        frame: self.frame + 1,
                        action: TestAction::SelectDropdownOption(test_id, option_index),
                    });
                    self.actions.sort_by_key(|action| action.frame);
                    return;
                }
                let entity = stage.accessibility.test_id_map.get(&test_id).copied();
                if let Some(entity) = entity {
                    let option_count = if let Some(data) =
                        storage.get::<nightshade_ui::components::UiDropdownData>(entity)
                    {
                        data.options.len()
                    } else {
                        0
                    };
                    if option_count > 0 {
                        if let Some(center) = self.resolve_entity_center(storage, stage, &test_id) {
                            self.inject_click_at(stage, center);
                        }
                        let nav_start = self.frame + 2;
                        let arrow_count = if option_index == 0 {
                            option_count
                        } else {
                            option_index
                        };
                        for step in 0..arrow_count {
                            self.actions.push(ScheduledAction {
                                frame: nav_start + step as u64,
                                action: TestAction::PressAndReleaseKey(KeyCode::ArrowDown),
                            });
                        }
                        self.actions.push(ScheduledAction {
                            frame: nav_start + arrow_count as u64,
                            action: TestAction::PressAndReleaseKey(KeyCode::Enter),
                        });
                        self.actions.sort_by_key(|action| action.frame);
                    } else {
                        eprintln!(
                            "[UI_TEST] WARNING: '{test_id}' is not a dropdown or has no options"
                        );
                    }
                } else {
                    eprintln!(
                        "[UI_TEST] WARNING: entity '{test_id}' not found for select_dropdown_option"
                    );
                }
            }
            TestAction::SelectTab(test_id, tab_index) => {
                if self.scroll_into_view(storage, stage, &test_id) {
                    self.actions.push(ScheduledAction {
                        frame: self.frame + 1,
                        action: TestAction::SelectTab(test_id, tab_index),
                    });
                    self.actions.sort_by_key(|action| action.frame);
                    return;
                }
                let entity = stage.accessibility.test_id_map.get(&test_id).copied();
                if let Some(entity) = entity {
                    let tab_entity = if let Some(data) =
                        storage.get::<nightshade_ui::components::UiTabBarData>(entity)
                    {
                        data.tab_entities.get(tab_index).copied()
                    } else {
                        None
                    };
                    if let Some(tab_entity) = tab_entity {
                        if let Some(center) = self.resolve_raw_entity_center(storage, tab_entity) {
                            self.inject_click_at(stage, center);
                        } else {
                            eprintln!(
                                "[UI_TEST] WARNING: tab {tab_index} in '{test_id}' has no layout"
                            );
                        }
                    } else {
                        eprintln!(
                            "[UI_TEST] WARNING: '{test_id}' is not a tab bar or index {tab_index} out of range"
                        );
                    }
                } else {
                    eprintln!("[UI_TEST] WARNING: entity '{test_id}' not found for select_tab");
                }
            }
            TestAction::TypeChars(chars) => {
                for character in &chars {
                    stage.input.keyboard.frame_chars.push(*character);
                }
            }
            TestAction::PressAndReleaseKey(key) => {
                stage
                    .input
                    .keyboard
                    .keystates
                    .insert(key, nightshade_platform::ElementState::Pressed);
                stage.input.keyboard.just_pressed_keys.insert(key);
                stage.input.keyboard.frame_keys.push((key, true));
                self.pending_key_releases.push(key);
            }
            TestAction::Scroll(delta) => {
                stage.input.mouse.wheel_delta = delta;
                stage.input.mouse.state.insert(MouseState::SCROLLED);
            }
            TestAction::Assert(name, kind) => {
                let result = self.evaluate_assertion(storage, stage, &name, &kind);
                self.result_frames.push(self.frame);
                self.results.push(result);
            }
            TestAction::Log(message) => {
                eprintln!("[UI_TEST] {message}");
            }
        }
    }

    fn evaluate_assertion(
        &self,
        storage: &nightshade::ecs::Storage,
        stage: &crate::types::TestStage<'_>,
        name: &str,
        kind: &AssertKind,
    ) -> UiTestResult {
        match kind {
            AssertKind::Visible(test_id) => {
                let entity = stage.accessibility.test_id_map.get(test_id);
                let visible = entity
                    .and_then(|entity| {
                        storage.get::<nightshade_ui::components::UiLayoutNode>(*entity)
                    })
                    .is_some_and(|node| node.visible);
                UiTestResult {
                    name: name.to_string(),
                    passed: visible,
                    message: if visible {
                        format!("'{test_id}' is visible")
                    } else {
                        format!("'{test_id}' expected visible but was not")
                    },
                }
            }
            AssertKind::NotVisible(test_id) => {
                let entity = stage.accessibility.test_id_map.get(test_id);
                let visible = entity
                    .and_then(|entity| {
                        storage.get::<nightshade_ui::components::UiLayoutNode>(*entity)
                    })
                    .is_some_and(|node| node.visible);
                UiTestResult {
                    name: name.to_string(),
                    passed: !visible,
                    message: if !visible {
                        format!("'{test_id}' is not visible")
                    } else {
                        format!("'{test_id}' expected not visible but was visible")
                    },
                }
            }
            AssertKind::WidgetValueF32(test_id, expected, tolerance) => {
                match read_widget_f32(storage, stage, test_id) {
                    Some(actual) => {
                        let passed = (actual - expected).abs() <= *tolerance;
                        UiTestResult {
                            name: name.to_string(),
                            passed,
                            message: if passed {
                                format!("'{test_id}' value {actual} matches expected {expected}")
                            } else {
                                format!(
                                    "'{test_id}' expected {expected} (tolerance {tolerance}), got {actual}"
                                )
                            },
                        }
                    }
                    None => UiTestResult {
                        name: name.to_string(),
                        passed: false,
                        message: format!("'{test_id}' has no f32 widget value"),
                    },
                }
            }
            AssertKind::WidgetValueBool(test_id, expected) => {
                match read_widget_bool(storage, stage, test_id) {
                    Some(actual) => {
                        let passed = actual == *expected;
                        UiTestResult {
                            name: name.to_string(),
                            passed,
                            message: if passed {
                                format!("'{test_id}' is {expected}")
                            } else {
                                format!("'{test_id}' expected {expected}, got {actual}")
                            },
                        }
                    }
                    None => UiTestResult {
                        name: name.to_string(),
                        passed: false,
                        message: format!("'{test_id}' has no bool widget value"),
                    },
                }
            }
            AssertKind::EventFired(event_match) => {
                let found = self.captured_events.iter().any(|event| match event_match {
                    EventMatch::ButtonClicked => matches!(event, UiEvent::ButtonClicked(_)),
                    EventMatch::SliderChanged => matches!(event, UiEvent::SliderChanged { .. }),
                    EventMatch::ToggleChanged => matches!(event, UiEvent::ToggleChanged { .. }),
                    EventMatch::TextInputChanged => {
                        matches!(event, UiEvent::TextInputChanged { .. })
                    }
                    EventMatch::CheckboxChanged => {
                        matches!(event, UiEvent::CheckboxChanged { .. })
                    }
                    EventMatch::DropdownChanged => {
                        matches!(event, UiEvent::DropdownChanged { .. })
                    }
                    EventMatch::RadioChanged => matches!(event, UiEvent::RadioChanged { .. }),
                    EventMatch::TabChanged => matches!(event, UiEvent::TabChanged { .. }),
                    EventMatch::SelectableLabelClicked => {
                        matches!(event, UiEvent::SelectableLabelClicked { .. })
                    }
                });
                UiTestResult {
                    name: name.to_string(),
                    passed: found,
                    message: if found {
                        format!("Event {event_match:?} was fired")
                    } else {
                        format!("Event {event_match:?} expected but not fired")
                    },
                }
            }
        }
    }

    fn record_action(
        &mut self,
        action: &TestAction,
        storage: &nightshade::ecs::Storage,
        stage: &crate::types::TestStage<'_>,
    ) {
        let (kind, target, position) = match action {
            TestAction::MoveMouseToEntity(id) => (
                "move_mouse_to_entity",
                Some(id.clone()),
                self.resolve_entity_center(storage, stage, id),
            ),
            TestAction::ClickAtEntity(id) => (
                "click_at_entity",
                Some(id.clone()),
                self.resolve_entity_center(storage, stage, id),
            ),
            TestAction::SetSliderValue(id, _) => ("set_slider_value", Some(id.clone()), None),
            TestAction::SelectDropdownOption(id, _) => {
                ("select_dropdown_option", Some(id.clone()), None)
            }
            TestAction::SelectTab(id, _) => ("select_tab", Some(id.clone()), None),
            TestAction::TypeChars(_) => ("type_chars", None, None),
            TestAction::PressAndReleaseKey(_) => ("press_and_release_key", None, None),
            TestAction::Scroll(p) => ("scroll", None, Some(*p)),
            TestAction::Assert(name, _) => ("assert", Some(name.clone()), None),
            TestAction::Log(msg) => ("log", Some(msg.clone()), None),
        };
        self.action_records.push(ActionRecord {
            frame: self.frame,
            kind: kind.to_string(),
            target,
            position: position.map(|point| [point.x, point.y]),
        });
    }

    pub(super) fn write_json_report(&self) {
        let Some(path) = &self.json_output_path else {
            return;
        };
        let report = TestReport {
            test_name: self.test_name.clone(),
            passed: self.all_passed(),
            summary: ReportSummary {
                total: self.results.len(),
                passed: self.results.iter().filter(|result| result.passed).count(),
                failed: self.results.iter().filter(|result| !result.passed).count(),
            },
            results: self
                .results
                .iter()
                .enumerate()
                .map(|(index, result)| ResultRecord {
                    name: result.name.clone(),
                    passed: result.passed,
                    message: result.message.clone(),
                    frame: self.result_frames.get(index).copied().unwrap_or(0),
                })
                .collect(),
            actions: self.action_records.clone(),
            screenshots: self.screenshots.clone(),
        };
        match serde_json::to_string_pretty(&report) {
            Ok(json) => {
                if let Err(error) = std::fs::write(path, &json) {
                    eprintln!("[UI_TEST] failed to write JSON report to {path}: {error}");
                }
            }
            Err(error) => {
                eprintln!("[UI_TEST] failed to serialize JSON report: {error}");
            }
        }
    }

    pub(super) fn print_results(&self) {
        self.write_json_report();
        eprintln!();
        eprintln!("=== UI Test: {} ===", self.test_name);
        let mut pass_count = 0;
        let mut fail_count = 0;
        for result in &self.results {
            if result.passed {
                pass_count += 1;
                eprintln!("  PASS: {} - {}", result.name, result.message);
            } else {
                fail_count += 1;
                eprintln!("  FAIL: {} - {}", result.name, result.message);
            }
        }
        eprintln!();
        eprintln!(
            "Results: {} passed, {} failed, {} total",
            pass_count,
            fail_count,
            self.results.len()
        );
        if fail_count == 0 {
            eprintln!("ALL TESTS PASSED");
        } else {
            eprintln!("TESTS FAILED");
        }
        eprintln!("========================");
        eprintln!();
    }
}

fn entity_for_test_id(
    stage: &crate::types::TestStage<'_>,
    test_id: &str,
) -> Option<nightshade::ecs::Entity> {
    stage.accessibility.test_id_map.get(test_id).copied()
}

fn read_widget_f32(
    storage: &nightshade::ecs::Storage,
    stage: &crate::types::TestStage<'_>,
    test_id: &str,
) -> Option<f32> {
    let entity = entity_for_test_id(stage, test_id)?;
    if let Some(data) = storage.get::<nightshade_ui::components::UiSliderData>(entity) {
        return Some(data.value);
    }
    if let Some(data) = storage.get::<nightshade_ui::components::UiDragValueData>(entity) {
        return Some(data.value);
    }
    if let Some(data) = storage.get::<nightshade_ui::components::UiProgressBarData>(entity) {
        return Some(data.value);
    }
    None
}

fn read_widget_bool(
    storage: &nightshade::ecs::Storage,
    stage: &crate::types::TestStage<'_>,
    test_id: &str,
) -> Option<bool> {
    let entity = entity_for_test_id(stage, test_id)?;
    if let Some(data) = storage.get::<nightshade_ui::components::UiToggleData>(entity) {
        return Some(data.value);
    }
    if let Some(data) = storage.get::<nightshade_ui::components::UiCheckboxData>(entity) {
        return Some(data.value);
    }
    if let Some(data) = storage.get::<nightshade_ui::components::UiSelectableLabelData>(entity) {
        return Some(data.selected);
    }
    if let Some(data) = storage.get::<nightshade_ui::components::UiRadioData>(entity) {
        return Some(data.selected);
    }
    if let Some(data) = storage.get::<nightshade_ui::components::UiCollapsingHeaderData>(entity) {
        return Some(data.open);
    }
    None
}
