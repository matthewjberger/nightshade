pub struct Person {
    pub col: i32,
    pub row: i32,
    pub upper: bool,
    pub name: &'static str,
    pub lines: &'static [&'static str],
}

const fn person(
    col: i32,
    row: i32,
    upper: bool,
    name: &'static str,
    lines: &'static [&'static str],
) -> Person {
    Person {
        col,
        row,
        upper,
        name,
        lines,
    }
}

pub const PEOPLE: &[Person] = &[
    person(
        26,
        33,
        false,
        "A man at the phone box",
        &[
            "The line is open. It has been open since Tuesday.",
            "There is nobody on it. There is a sound on it.",
            "If you listen long enough the sound learns your name and uses it.",
            "I am going to stand here and not listen. You should keep walking.",
        ],
    ),
    person(
        16,
        22,
        false,
        "A man in the cells",
        &[
            "Do not open this door. I asked them to lock it and they did.",
            "There is one of them in the back that is bigger than the others.",
            "It does not run. It does not have to. Do not let it get its hands on you.",
            "The brass key is in the evidence room. Take it and do not come back this way.",
        ],
    ),
    person(
        43,
        22,
        false,
        "A nurse at reception",
        &[
            "You are not hurt. Everybody who walks in here is not hurt.",
            "Count the doors in the ward corridor. There are eleven of them.",
            "There have only ever been ten. Do not open the one that should not be there.",
            "The ward key is in the pharmacy at the back, shut in a dispensary box.",
            "There is no keyhole on that box. It comes open from underneath.",
        ],
    ),
    person(
        51,
        25,
        false,
        "A boy in the corridor",
        &[
            "I am not allowed to look at you. Sorry.",
            "The bell goes at four and then it is the other school.",
            "Mum went to the other school to get my sister. That was eleven years ago.",
            "The library is that way. There is a key in there made out of a person.",
        ],
    ),
    person(
        46,
        11,
        false,
        "The verger",
        &[
            "You have come the long way round. Everybody does.",
            "The gate wants a key of bone, and the key is in two pieces.",
            "A better man than you broke it so that nobody could carry it alone.",
            "Put it back together and the town turns over and shows you its other side.",
            "Do not stop to look at it. Walk to the gate. Do not answer anything that calls you.",
        ],
    ),
    person(
        14,
        42,
        true,
        "A woman on the landing",
        &[
            "I keep the light off. You should keep yours off as well.",
            "They see the beam a long way further than they see you.",
            "When the cell in your torch dies, that is not the worst thing that can happen.",
            "There are cells in the drawer by the window. Take them. I have no use left for them.",
        ],
    ),
];
