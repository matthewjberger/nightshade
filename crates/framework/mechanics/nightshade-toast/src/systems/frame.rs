use crate::resources::Toast;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;

pub fn fade_the_toast(time: Res<Time>, mut toast: ResMut<Toast>) {
    if toast.remaining <= 0.0 {
        return;
    }
    toast.remaining = (toast.remaining - time.delta_time).max(0.0);
    if toast.remaining <= 0.0 {
        toast.line.clear();
    }
}
