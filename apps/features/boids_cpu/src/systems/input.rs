use crate::resources::BoidsResources;
use crate::systems::population::stepped_count;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn handle_events(mut resources: ResMut<BoidsResources>, reader: EventReader<AppEvent>) {
    for event in reader.iter() {
        if let AppEvent::Keyboard {
            key,
            state: KeyState::Pressed,
        } = event
        {
            match key {
                KeyCode::Digit1 => resources.show_headings = !resources.show_headings,
                KeyCode::Digit2 => resources.show_bounds = !resources.show_bounds,
                KeyCode::Digit3 => resources.show_neighbors = !resources.show_neighbors,
                KeyCode::Digit4 => resources.show_boids = !resources.show_boids,
                KeyCode::ArrowRight => {
                    resources.requested_count = stepped_count(resources.requested_count, 1);
                }
                KeyCode::ArrowLeft => {
                    resources.requested_count = stepped_count(resources.requested_count, -1);
                }
                _ => {}
            }
        }
    }
}
