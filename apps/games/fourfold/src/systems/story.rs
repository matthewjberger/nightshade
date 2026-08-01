use crate::data;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;

pub fn tell(time: Res<Time>, mut resources: ResMut<FourfoldResources>) {
    let delta_time = time.delta_time;
    if resources.fade > 0.0 {
        resources.fade = (resources.fade - delta_time / data::FADE_SECONDS).max(0.0);
    }

    let story = &mut resources.story;
    match story.showing {
        Some(index) => {
            story.held += delta_time;
            if story.held >= data::SCRIPT[index].hold {
                story.showing = None;
                story.held = 0.0;
            }
        }
        None => {
            if story.queue.is_empty() {
                return;
            }
            story.showing = Some(story.queue.remove(0));
            story.held = 0.0;
        }
    }
}
