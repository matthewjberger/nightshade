#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spot {
    WatchLodge,
    BarrowMouth,
    BarrowDeep,
    WoodsCamp,
}

pub struct Place {
    pub spot: Spot,
    pub name: &'static str,
    pub reach: f32,
}

pub const PLACES: [Place; 4] = [
    place(Spot::WatchLodge, "the watch lodge", 6.0),
    place(Spot::BarrowMouth, "the barrow mouth", 10.0),
    place(Spot::BarrowDeep, "the deep chamber", 5.0),
    place(Spot::WoodsCamp, "the camp in the trees", 9.0),
];

const fn place(spot: Spot, name: &'static str, reach: f32) -> Place {
    Place { spot, name, reach }
}

pub const WATCH_LODGE: usize = 0;

pub const BARROW_MOUTH: usize = 1;

pub const BARROW_DEEP: usize = 2;

pub const WOODS_CAMP: usize = 3;

const fn places_line_up() -> bool {
    matches!(PLACES[WATCH_LODGE].spot, Spot::WatchLodge)
        && matches!(PLACES[BARROW_MOUTH].spot, Spot::BarrowMouth)
        && matches!(PLACES[BARROW_DEEP].spot, Spot::BarrowDeep)
        && matches!(PLACES[WOODS_CAMP].spot, Spot::WoodsCamp)
}

const _: () = assert!(places_line_up());
