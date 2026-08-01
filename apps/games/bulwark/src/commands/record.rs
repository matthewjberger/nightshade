use crate::data::Record;
use crate::queries;
use crate::resources::Bulwark;
use crate::theme;
use nightshade_save::prelude::save_write;
use nightshade_waves::WaveProgress;

pub fn keep_the_record(bulwark: &mut Bulwark, progress: &WaveProgress) -> bool {
    let held = queries::waves_held(bulwark, progress);
    if held <= bulwark.best {
        return false;
    }
    bulwark.best = held;
    let label = queries::best_line(held);
    save_write(theme::RECORD_SLOT, &label, &Record { best: held });
    true
}
