use blocker_screen::data::names::same_name;

pub struct Character {
    pub letter: &'static str,
    pub model: &'static [u8],
    pub skin_path: &'static str,
    pub skin: &'static [u8],
}

macro_rules! character {
    ($letter:literal) => {
        Character {
            letter: $letter,
            model: include_bytes!(concat!(
                "../../../../assets/characters/character-",
                $letter,
                ".glb"
            )),
            skin_path: concat!("Textures/texture-", $letter, ".png"),
            skin: include_bytes!(concat!(
                "../../../../assets/characters/Textures/texture-",
                $letter,
                ".png"
            )),
        }
    };
}

pub const CHARACTERS: [Character; 18] = [
    character!("a"),
    character!("b"),
    character!("c"),
    character!("d"),
    character!("e"),
    character!("f"),
    character!("g"),
    character!("h"),
    character!("i"),
    character!("j"),
    character!("k"),
    character!("l"),
    character!("m"),
    character!("n"),
    character!("o"),
    character!("p"),
    character!("q"),
    character!("r"),
];

pub const ROTTEN_LOOK: usize = look_named("o");

pub const fn look_named(letter: &str) -> usize {
    let mut index = 0;
    while index < CHARACTERS.len() {
        if same_name(CHARACTERS[index].letter, letter) {
            return index;
        }
        index += 1;
    }
    panic!("the character kit holds no figure with that letter")
}

pub const LIVING_LOOKS: [usize; 17] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17];

const fn looks_leave_out_the_rotten() -> bool {
    let mut index = 0;
    while index < LIVING_LOOKS.len() {
        if LIVING_LOOKS[index] == ROTTEN_LOOK {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(looks_leave_out_the_rotten());
const _: () = assert!(ROTTEN_LOOK < CHARACTERS.len());
