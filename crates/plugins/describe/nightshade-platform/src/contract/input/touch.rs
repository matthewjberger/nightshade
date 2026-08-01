use std::collections::HashMap;

#[derive(Default, Debug, Clone, Copy)]
pub struct TouchPoint {
    pub id: u64,
    pub position: nalgebra_glm::Vec2,
    pub start_position: nalgebra_glm::Vec2,
    pub previous_position: nalgebra_glm::Vec2,
    pub phase: TouchPhase,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    #[default]
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Default, Debug, Clone)]
pub struct Touch {
    pub touches: HashMap<u64, TouchPoint>,
    pub primary_touch_id: Option<u64>,
    pub secondary_touch_id: Option<u64>,
    pub gesture: TouchGesture,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum TouchGesture {
    #[default]
    None,
    SingleDrag {
        delta: nalgebra_glm::Vec2,
    },
    Pinch {
        delta: f32,
        center: nalgebra_glm::Vec2,
    },
    TwoFingerDrag {
        delta: nalgebra_glm::Vec2,
    },
}

impl Touch {
    pub fn active_touch_count(&self) -> usize {
        self.touches
            .values()
            .filter(|t| t.phase != TouchPhase::Ended && t.phase != TouchPhase::Cancelled)
            .count()
    }

    pub fn primary_touch(&self) -> Option<&TouchPoint> {
        self.primary_touch_id
            .and_then(|id| self.touches.get(&id))
            .filter(|t| t.phase != TouchPhase::Ended && t.phase != TouchPhase::Cancelled)
    }

    pub fn secondary_touch(&self) -> Option<&TouchPoint> {
        self.secondary_touch_id
            .and_then(|id| self.touches.get(&id))
            .filter(|t| t.phase != TouchPhase::Ended && t.phase != TouchPhase::Cancelled)
    }

    pub fn update_gesture(&mut self) {
        let active_count = self.active_touch_count();

        self.gesture = match active_count {
            1 => {
                if let Some(touch) = self.primary_touch() {
                    if touch.phase == TouchPhase::Moved {
                        let delta = touch.position - touch.previous_position;
                        TouchGesture::SingleDrag { delta }
                    } else {
                        TouchGesture::None
                    }
                } else {
                    TouchGesture::None
                }
            }
            2 => {
                if let (Some(primary), Some(secondary)) =
                    (self.primary_touch(), self.secondary_touch())
                {
                    let current_distance = (primary.position - secondary.position).magnitude();
                    let previous_distance =
                        (primary.previous_position - secondary.previous_position).magnitude();
                    let pinch_delta = current_distance - previous_distance;

                    let primary_delta = primary.position - primary.previous_position;
                    let secondary_delta = secondary.position - secondary.previous_position;
                    let avg_delta = (primary_delta + secondary_delta) * 0.5;

                    let center = (primary.position + secondary.position) * 0.5;

                    if pinch_delta.abs() > avg_delta.magnitude() * 0.5 {
                        TouchGesture::Pinch {
                            delta: pinch_delta,
                            center,
                        }
                    } else {
                        TouchGesture::TwoFingerDrag { delta: avg_delta }
                    }
                } else {
                    TouchGesture::None
                }
            }
            _ => TouchGesture::None,
        };
    }

    pub fn end_frame(&mut self) {
        self.clear_ended_touches();
        self.gesture = TouchGesture::None;
    }

    pub fn clear_ended_touches(&mut self) {
        self.touches
            .retain(|_, t| t.phase != TouchPhase::Ended && t.phase != TouchPhase::Cancelled);

        if let Some(id) = self.primary_touch_id
            && !self.touches.contains_key(&id)
        {
            self.primary_touch_id = None;
        }
        if let Some(id) = self.secondary_touch_id
            && !self.touches.contains_key(&id)
        {
            self.secondary_touch_id = None;
        }
    }
}
