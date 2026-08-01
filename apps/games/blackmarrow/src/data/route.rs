use nightshade::prelude::Vec3;

pub struct Waypoint {
    pub at: Vec3,
    pub links: &'static [usize],
}

pub const WOLF_DEN: usize = 6;

pub const WAYPOINTS: &[Waypoint] = &[
    node(4.0, 3.2, &[1]),
    node(4.0, 7.0, &[0, 2]),
    node(3.0, 8.5, &[1, 3, 5]),
    node(8.0, 8.5, &[2, 4]),
    node(12.5, 8.5, &[3, 11]),
    node(3.0, 10.0, &[2, 6]),
    node(3.4, 12.6, &[5, 7]),
    node(3.4, 17.0, &[6, 8]),
    node(3.0, 19.0, &[7, 9]),
    node(3.5, 22.4, &[8, 10]),
    node(3.5, 24.2, &[9]),
    node(12.5, 10.0, &[4, 12]),
    node(12.5, 12.4, &[11, 13]),
    node(12.5, 15.0, &[12, 14]),
    node(12.5, 18.2, &[13]),
];

const fn node(x: f32, z: f32, links: &'static [usize]) -> Waypoint {
    Waypoint {
        at: Vec3::new(x, 0.0, z),
        links,
    }
}
