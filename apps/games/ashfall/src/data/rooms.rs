use crate::data::level::{HEIGHT, WIDTH};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomKind {
    Street,
    Library,
    Church,
    Gate,
    Police,
    Cells,
    Clinic,
    Pharmacy,
    Home,
    Stairwell,
    Market,
    Office,
    Open,
}

pub struct Room {
    pub name: &'static str,
    pub col0: i32,
    pub row0: i32,
    pub col1: i32,
    pub row1: i32,
    pub upper: bool,
    pub kind: RoomKind,
}

const fn room(
    name: &'static str,
    col0: i32,
    row0: i32,
    col1: i32,
    row1: i32,
    upper: bool,
    kind: RoomKind,
) -> Room {
    Room {
        name,
        col0,
        row0,
        col1,
        row1,
        upper,
        kind,
    }
}

pub const ROOMS: &[Room] = &[
    room("Mercy Street", 2, 32, 61, 35, false, RoomKind::Street),
    room("North Row", 2, 14, 61, 17, false, RoomKind::Street),
    room("Cross Street", 30, 2, 33, 45, false, RoomKind::Street),
    room("Ash Lane", 46, 19, 47, 31, false, RoomKind::Street),
    room("The Stacks", 2, 2, 15, 7, false, RoomKind::Library),
    room("Schoolroom", 17, 2, 28, 7, false, RoomKind::Library),
    room("Reading Hall", 2, 9, 28, 12, false, RoomKind::Library),
    room("Vestry", 35, 2, 40, 5, false, RoomKind::Church),
    room("Chancel", 41, 2, 55, 5, false, RoomKind::Church),
    room("Nave", 35, 7, 55, 12, false, RoomKind::Church),
    room("Gate Chamber", 57, 2, 61, 12, false, RoomKind::Gate),
    room("Front Desk", 2, 19, 13, 30, false, RoomKind::Police),
    room("Cells", 15, 19, 21, 23, false, RoomKind::Cells),
    room("Cell Block", 15, 25, 21, 30, false, RoomKind::Cells),
    room("Evidence Room", 23, 19, 28, 30, false, RoomKind::Police),
    room("Reception", 35, 19, 45, 23, false, RoomKind::Clinic),
    room("Ward", 35, 25, 41, 30, false, RoomKind::Clinic),
    room("Pharmacy", 43, 25, 45, 30, false, RoomKind::Pharmacy),
    room("Almshouse Hall", 49, 19, 54, 30, false, RoomKind::Home),
    room("Almshouse Rooms", 56, 19, 61, 30, false, RoomKind::Home),
    room("Stairwell", 2, 37, 7, 45, false, RoomKind::Stairwell),
    room("Tenement Rooms", 9, 37, 17, 45, false, RoomKind::Home),
    room("Back Rooms", 19, 37, 28, 45, false, RoomKind::Home),
    room("Market Floor", 35, 37, 51, 45, false, RoomKind::Market),
    room("Market Office", 53, 37, 61, 45, false, RoomKind::Office),
    room("Gallery", 2, 9, 28, 12, true, RoomKind::Library),
    room("Records Loft", 2, 19, 21, 30, true, RoomKind::Office),
    room("Loft Store", 23, 19, 28, 30, true, RoomKind::Office),
    room("Upper Ward", 35, 19, 45, 24, true, RoomKind::Clinic),
    room("Sick Rooms", 35, 26, 45, 30, true, RoomKind::Clinic),
    room("Landing", 2, 37, 17, 45, true, RoomKind::Home),
    room("Upper Rooms", 19, 37, 28, 45, true, RoomKind::Home),
    room("Balcony", 9, 36, 14, 36, true, RoomKind::Open),
    room("Catwalk", 11, 31, 12, 35, true, RoomKind::Open),
];

const _: () = {
    let mut index = 0;
    while index < ROOMS.len() {
        assert!(ROOMS[index].col0 >= 0 && ROOMS[index].col1 < WIDTH as i32);
        assert!(ROOMS[index].row0 >= 0 && ROOMS[index].row1 < HEIGHT as i32);
        index += 1;
    }
};
