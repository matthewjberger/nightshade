use crate::data::team_tile;
use crate::queries::Structures;
use crate::resources::Sheet;
use crate::theme;
use nightshade::prelude::*;
use nightshade_sprite::prelude::*;

pub fn paint_the_structures(
    sheet: Res<Sheet>,
    mut structures: Structures<'_, '_>,
    mut flags: Query<(&mut SpriteAnimator,)>,
) {
    let mut banners = Vec::new();
    structures.for_each_mut(|_entity, (_, holder, _, _, flag)| {
        if let Some(banner) = flag.0 {
            banners.push((banner, holder.0));
        }
    });
    for (flag, owner) in banners {
        let still = crate::queries::sheet::standing(&sheet, team_tile(theme::FLAG_TILE, owner)).0;
        let waving =
            crate::queries::sheet::standing(&sheet, team_tile(theme::FLAG_WAVE_TILE, owner)).0;
        flags.for_entity(flag, |(animator,)| {
            if animator.frames.first().map(|frame| frame.region) != Some(still) {
                animator.frames = vec![
                    SpriteAnimationFrame {
                        region: still,
                        duration: theme::FLAG_FRAME_SECONDS,
                    },
                    SpriteAnimationFrame {
                        region: waving,
                        duration: theme::FLAG_FRAME_SECONDS,
                    },
                ];
            }
        });
    }
}
