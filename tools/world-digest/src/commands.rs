use crate::queries::{column_count, column_digest, world_digest};
use crate::resources::Run;

pub fn report_the_digest(run: &Run) {
    let (world, columns) = match run.game.as_str() {
        "voxels" => voxels::digest_the_world(run.seed, run.radius),
        "hollowmere" => hollowmere::digest_the_world(run.seed, run.radius),
        _ => return,
    };
    println!(
        "game {} seed {} radius {} columns {}",
        run.game,
        run.seed,
        run.radius,
        column_count(run.radius)
    );
    println!("total {:016x}", world_digest(&world, &columns));
    for column in columns {
        let digest = column_digest(&world, column);
        println!("  [{:>4},{:>4}] {digest:016x}", column[0], column[1]);
    }
}
