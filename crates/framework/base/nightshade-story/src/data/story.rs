#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Place {
    #[default]
    Gone,
    Room(usize),
    Held,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Above,
    Below,
    Same,
    Apart,
    Least,
    Most,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Set,
    Add,
    Subtract,
}

#[derive(Clone, Debug)]
pub enum Test {
    Holds(usize),
    At(usize),
    Here(usize),
    Value {
        name: String,
        sign: Sign,
        amount: i32,
    },
    Not(Box<Test>),
    All(Vec<Test>),
    Any(Vec<Test>),
}

#[derive(Clone, Debug)]
pub enum Change {
    Move {
        thing: usize,
        place: Place,
    },
    Value {
        name: String,
        step: Step,
        amount: i32,
    },
    Go(usize),
    End(String),
}

#[derive(Clone, Debug, Default)]
pub struct Way {
    pub word: String,
    pub to: usize,
    pub when: Option<Test>,
    pub barred: String,
}

#[derive(Clone, Debug, Default)]
pub struct Room {
    pub name: String,
    pub title: String,
    pub look: String,
    pub ways: Vec<Way>,
}

#[derive(Clone, Debug, Default)]
pub struct Thing {
    pub name: String,
    pub words: Vec<String>,
    pub look: String,
    pub start: Place,
    pub portable: bool,
    pub taking: String,
}

#[derive(Clone, Debug, Default)]
pub struct Rule {
    pub verb: String,
    pub thing: Option<usize>,
    pub when: Option<Test>,
    pub say: String,
    pub barred: String,
    pub changes: Vec<Change>,
}

#[derive(Clone, Debug, Default)]
pub struct Story {
    pub title: String,
    pub opening: Vec<String>,
    pub start: usize,
    pub rooms: Vec<Room>,
    pub things: Vec<Thing>,
    pub rules: Vec<Rule>,
    pub values: Vec<(String, i32)>,
}
