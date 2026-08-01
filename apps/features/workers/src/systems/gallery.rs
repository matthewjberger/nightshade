use crate::commands::sculpt;
use crate::commands::stand_a_statue;
use crate::data::{Chisel, SculptJob, SculptWork, Stage};
use crate::resources::Gallery;
use crate::theme;
use nightshade::prelude::*;
use nightshade_render::prelude::MeshCache;
use nightshade_worker::{Job, Workers, workers_submit, workers_take};

impl Job for SculptJob {
    type Key = usize;
    type Done = SculptWork;
    type Tools = Chisel;

    fn key(&self) -> Self::Key {
        self.slot
    }

    fn run(self, tools: &mut Self::Tools) -> Self::Done {
        SculptWork {
            slot: self.slot,
            mesh: sculpt(self.seed, self.slot, tools),
            era: self.era,
        }
    }
}

pub fn ask_the_crew(mut gallery: ResMut<Gallery>, mut crew: ResMut<Workers<SculptJob>>) {
    if gallery.paused || gallery.on_the_frame {
        return;
    }
    let era = gallery.era;
    for slot in 0..theme::STANDS {
        if gallery.stage[slot] != Stage::Empty {
            continue;
        }
        let job = SculptJob {
            slot,
            seed: seed_of(era, slot),
            era,
        };
        if workers_submit(&mut crew, job, slot as i32) {
            gallery.stage[slot] = Stage::Asked;
        }
    }
}

pub fn take_the_statues(
    mut gallery: ResMut<Gallery>,
    mut crew: ResMut<Workers<SculptJob>>,
    mut mesh_cache: ResMut<MeshCache>,
    mut commands: Commands,
) {
    let finished = workers_take(&mut crew, theme::TAKE_LIMIT);
    for work in finished {
        if work.era != gallery.era {
            continue;
        }
        stand_a_statue(&mut mesh_cache, &mut commands, work.slot, work.mesh);
        gallery.stage[work.slot] = Stage::Standing;
    }
}

pub fn toil_on_the_frame(
    mut gallery: ResMut<Gallery>,
    mut mesh_cache: ResMut<MeshCache>,
    mut commands: Commands,
) {
    if gallery.paused || !gallery.on_the_frame {
        return;
    }
    let Some(slot) = (0..theme::STANDS).find(|slot| gallery.stage[*slot] == Stage::Empty) else {
        return;
    };
    let seed = seed_of(gallery.era, slot);
    let mut chisel = std::mem::take(&mut gallery.chisel);
    let mesh = sculpt(seed, slot, &mut chisel);
    gallery.chisel = chisel;
    stand_a_statue(&mut mesh_cache, &mut commands, slot, mesh);
    gallery.stage[slot] = Stage::Standing;
}

fn seed_of(era: u32, slot: usize) -> u64 {
    era as u64 * theme::STANDS as u64 + slot as u64 + 1
}
