use nalgebra_glm;
use nightshade_paint::data::Rect;

pub struct PickingEntry {
    pub entity: nightshade::ecs::Entity,
    pub depth: f32,
    pub rotation: f32,
    pub pivot: nalgebra_glm::Vec2,
    pub computed_rect: Rect,
    pub clip_rect: Option<Rect>,
    pub disabled: bool,
}

pub struct PickingGrid {
    cells: Vec<Vec<PickingEntry>>,
    cell_size: f32,
    columns: usize,
    rows: usize,
}

impl PickingGrid {
    pub(crate) fn new(viewport_width: f32, viewport_height: f32, cell_size: f32) -> Self {
        let columns = ((viewport_width / cell_size).ceil() as usize).max(1);
        let rows = ((viewport_height / cell_size).ceil() as usize).max(1);
        Self {
            cells: (0..columns * rows).map(|_| Vec::new()).collect(),
            cell_size,
            columns,
            rows,
        }
    }

    pub(crate) fn insert(&mut self, entry: PickingEntry) {
        let effective_rect = if let Some(clip) = entry.clip_rect {
            match entry.computed_rect.intersect(&clip) {
                Some(clipped) => clipped,
                None => return,
            }
        } else {
            entry.computed_rect
        };
        let effective_rect = if entry.rotation == 0.0 {
            effective_rect
        } else {
            spun_bounds(effective_rect, entry.pivot, entry.rotation)
        };

        let epsilon = 0.01;
        let col_start =
            (((effective_rect.min.x - epsilon) / self.cell_size).floor() as isize).max(0) as usize;
        let col_end =
            (((effective_rect.max.x + epsilon) / self.cell_size).ceil() as usize).min(self.columns);
        let row_start =
            (((effective_rect.min.y - epsilon) / self.cell_size).floor() as isize).max(0) as usize;
        let row_end =
            (((effective_rect.max.y + epsilon) / self.cell_size).ceil() as usize).min(self.rows);

        for row in row_start..row_end {
            for col in col_start..col_end {
                let cell_index = row * self.columns + col;
                self.cells[cell_index].push(PickingEntry {
                    entity: entry.entity,
                    depth: entry.depth,
                    rotation: entry.rotation,
                    pivot: entry.pivot,
                    computed_rect: entry.computed_rect,
                    clip_rect: entry.clip_rect,
                    disabled: entry.disabled,
                });
            }
        }
    }

    pub(crate) fn sort_cells(&mut self) {
        for cell in &mut self.cells {
            cell.sort_by(|a, b| {
                b.depth
                    .partial_cmp(&a.depth)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    pub(crate) fn query(&self, point: nalgebra_glm::Vec2) -> Option<&[PickingEntry]> {
        let col = (point.x / self.cell_size).floor() as isize;
        let row = (point.y / self.cell_size).floor() as isize;
        if col < 0 || row < 0 || col >= self.columns as isize || row >= self.rows as isize {
            return None;
        }
        let cell_index = row as usize * self.columns + col as usize;
        Some(&self.cells[cell_index])
    }
}

fn spun_bounds(rect: Rect, pivot: nalgebra_glm::Vec2, angle: f32) -> Rect {
    let turn = angle.cos();
    let lean = angle.sin();
    let corners = [
        rect.min,
        nalgebra_glm::Vec2::new(rect.max.x, rect.min.y),
        rect.max,
        nalgebra_glm::Vec2::new(rect.min.x, rect.max.y),
    ];
    let mut bounds = Rect {
        min: nalgebra_glm::Vec2::new(f32::MAX, f32::MAX),
        max: nalgebra_glm::Vec2::new(f32::MIN, f32::MIN),
    };
    for corner in corners {
        let offset = corner - pivot;
        let spun = pivot
            + nalgebra_glm::Vec2::new(
                offset.x * turn - offset.y * lean,
                offset.x * lean + offset.y * turn,
            );
        bounds.min.x = bounds.min.x.min(spun.x);
        bounds.min.y = bounds.min.y.min(spun.y);
        bounds.max.x = bounds.max.x.max(spun.x);
        bounds.max.y = bounds.max.y.max(spun.y);
    }
    bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::TileId;
    use crate::resources::event::UiEvent;
    use nalgebra_glm::{Vec2, Vec4};

    fn dummy_entity(id: u32) -> nightshade::ecs::Entity {
        nightshade::ecs::Entity { id, generation: 0 }
    }

    #[test]
    fn button_clicked_bubbles() {
        assert!(UiEvent::ButtonClicked(dummy_entity(0)).bubbles());
    }

    #[test]
    fn slider_changed_bubbles() {
        assert!(
            UiEvent::SliderChanged {
                entity: dummy_entity(0),
                value: 0.0
            }
            .bubbles()
        );
    }

    #[test]
    fn tree_node_context_menu_does_not_bubble() {
        assert!(
            !UiEvent::TreeNodeContextMenu {
                tree: dummy_entity(0),
                node: dummy_entity(1),
                position: Vec2::zeros(),
            }
            .bubbles()
        );
    }

    #[test]
    fn modal_closed_does_not_bubble() {
        assert!(
            !UiEvent::ModalClosed {
                entity: dummy_entity(0),
                confirmed: true,
            }
            .bubbles()
        );
    }

    #[test]
    fn target_entity_button() {
        let entity = dummy_entity(42);
        assert_eq!(UiEvent::ButtonClicked(entity).target_entity(), entity);
    }

    #[test]
    fn target_entity_tree_node_selected() {
        let tree = dummy_entity(1);
        let node = dummy_entity(2);
        assert_eq!(
            UiEvent::TreeNodeSelected {
                tree,
                node,
                selected: true
            }
            .target_entity(),
            node
        );
    }

    #[test]
    fn target_entity_modal_closed() {
        let entity = dummy_entity(5);
        assert_eq!(
            UiEvent::ModalClosed {
                entity,
                confirmed: false
            }
            .target_entity(),
            entity
        );
    }

    #[test]
    fn all_bubbling_events_bubble() {
        let entity = dummy_entity(0);
        let bubbling_events = [
            UiEvent::ButtonClicked(entity),
            UiEvent::SliderChanged { entity, value: 0.0 },
            UiEvent::ToggleChanged {
                entity,
                value: false,
            },
            UiEvent::CheckboxChanged {
                entity,
                value: false,
            },
            UiEvent::RadioChanged {
                entity,
                group_id: 0,
                option_index: 0,
            },
            UiEvent::TabChanged {
                entity,
                tab_index: 0,
            },
            UiEvent::TextInputChanged {
                entity,
                text: String::new(),
            },
            UiEvent::TextInputSubmitted {
                entity,
                text: String::new(),
            },
            UiEvent::DropdownChanged {
                entity,
                selected_index: 0,
            },
            UiEvent::MenuItemClicked {
                entity,
                item_index: 0,
            },
            UiEvent::ColorPickerChanged {
                entity,
                color: Vec4::zeros(),
            },
            UiEvent::SelectableLabelClicked {
                entity,
                selected: false,
            },
            UiEvent::DragValueChanged { entity, value: 0.0 },
            UiEvent::ContextMenuItemClicked {
                entity,
                item_index: 0,
                tag: 0,
            },
            UiEvent::TreeNodeSelected {
                tree: entity,
                node: entity,
                selected: false,
            },
            UiEvent::TreeNodeToggled {
                tree: entity,
                node: entity,
                expanded: false,
            },
        ];
        for event in &bubbling_events {
            assert!(event.bubbles(), "Expected {:?} to bubble", event);
        }
    }

    #[test]
    fn all_non_bubbling_events_do_not_bubble() {
        let entity = dummy_entity(0);
        let non_bubbling_events = [
            UiEvent::TreeNodeContextMenu {
                tree: entity,
                node: entity,
                position: Vec2::zeros(),
            },
            UiEvent::ModalClosed {
                entity,
                confirmed: false,
            },
            UiEvent::CommandPaletteExecuted {
                entity,
                command_index: 0,
            },
            UiEvent::TileTabActivated {
                container: entity,
                pane_id: TileId(0),
            },
            UiEvent::TileTabClosed {
                container: entity,
                pane_id: TileId(0),
                title: String::new(),
            },
            UiEvent::TileSplitterMoved {
                container: entity,
                split_id: TileId(0),
                ratio: 0.5,
            },
        ];
        for event in &non_bubbling_events {
            assert!(!event.bubbles(), "Expected {:?} not to bubble", event);
        }
    }
}
