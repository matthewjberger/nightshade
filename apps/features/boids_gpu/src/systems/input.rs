use crate::queries::stepped_count;
use crate::resources::BoidsGpuResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn handle_events(mut resources: ResMut<BoidsGpuResources>, reader: EventReader<AppEvent>) {
    for event in reader.iter() {
        if let AppEvent::Keyboard {
            key,
            state: KeyState::Pressed,
        } = event
        {
            match key {
                KeyCode::Digit4 => resources.visible = !resources.visible,
                KeyCode::ArrowRight => resources.count = stepped_count(resources.count, 1),
                KeyCode::ArrowLeft => resources.count = stepped_count(resources.count, -1),
                _ => {}
            }
        }
    }
}
