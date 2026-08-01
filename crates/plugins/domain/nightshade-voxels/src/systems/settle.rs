use crate::commands::{finish_column, mark_column_dirty};
use crate::contract::ColumnKey;
use crate::data;
use crate::queries::{column_of, column_top};
use crate::resources::{ColumnJob, ColumnWork, Dig, VoxelSettings, VoxelViewer, VoxelWorld};
use nightshade::prelude::{Res, ResMut};
use nightshade_worker::prelude::*;
use std::sync::Arc;

pub fn settle_the_columns<D: Dig>(
    viewer: Res<VoxelViewer>,
    settings: Res<VoxelSettings>,
    mut world: ResMut<VoxelWorld>,
    mut orders: ResMut<D::Orders>,
    mut diggers: ResMut<Workers<ColumnJob<D>>>,
) {
    let finished = workers_take(&mut diggers, data::SETTLE_LIMIT);
    if finished.is_empty() {
        return;
    }
    let eye = viewer.eye / settings.block_size;
    let center = column_of(eye.x.floor() as i32, eye.z.floor() as i32);
    let keep = settings.keep_radius;
    for work in finished {
        if (work.column[0] - center[0]).abs() > keep || (work.column[1] - center[1]).abs() > keep {
            continue;
        }
        for (key, chunk) in work.chunks {
            world.chunks.insert(key, chunk);
        }
        world.tops.insert(work.column, column_top(&work.surface));
        world.surface.insert(work.column, work.surface);
        for (key, found) in work.markers {
            world.markers.entry(key).or_default().extend(found);
        }
        let mut touched = D::settle(&mut orders, &mut world, work.column, work.report);
        touched.push(work.column);
        touched.sort_unstable();
        touched.dedup();
        for column in touched {
            mark_column_dirty(&mut world, column);
        }
        world.sight += 1;
        world.edits += 1;
    }
}

impl<D: Dig> Job for ColumnJob<D> {
    type Key = ColumnKey;
    type Done = ColumnWork<D>;
    type Tools = D::Tools;

    fn key(&self) -> Self::Key {
        self.column
    }

    fn run(self, tools: &mut Self::Tools) -> Self::Done {
        let mut private = VoxelWorld {
            blocks: self.blocks,
            edited: self.edited,
            ..Default::default()
        };
        let report = D::dig(self.order, tools, &mut private, self.column);
        finish_column(&mut private, self.column);
        let surface = private
            .surface
            .remove(&self.column)
            .unwrap_or_else(|| Arc::new(vec![data::SURFACE_NONE; data::CHUNK_FACES]));
        ColumnWork {
            column: self.column,
            chunks: std::mem::take(&mut private.chunks),
            surface,
            markers: std::mem::take(&mut private.markers),
            report,
        }
    }
}
