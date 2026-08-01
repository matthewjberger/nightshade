pub const EMPTY: &str = "empty";

pub const WELCOME: &[&str] = &[
    "Build the game by clicking. Type a name in the box at the top of Build, then press the button that uses it.",
    "Each step below lights up when the description holds what it asks for.",
];

pub enum Want {
    Group(&'static str, &'static str),
    Values(&'static str, &'static str, usize),
    Component(&'static str),
    System(&'static str),
    Port(&'static str, &'static str),
    Spawns(&'static str, usize),
    Statements(&'static str, usize),
    Resource(&'static str),
}

pub struct Step {
    pub title: &'static str,
    pub note: &'static [&'static str],
    pub want: Want,
    pub answer: &'static str,
}

pub const SOURCES: &[(&str, &str)] = &[
    ("empty", include_str!("../../lessons/empty.ns")),
    ("shapes", include_str!("../../lessons/shapes.ns")),
    ("steering", include_str!("../../lessons/steering.ns")),
    ("bouncing", include_str!("../../lessons/bouncing.ns")),
    ("answering", include_str!("../../lessons/answering.ns")),
    ("bricks", include_str!("../../lessons/bricks.ns")),
];

pub const STEPS: &[Step] = &[
    Step {
        title: "Make a group for the court",
        note: &[
            "Open Build. In Numbers Game, type court in the name box and press add a group.",
            "A group keeps a few values together. No group holds more than five.",
        ],
        want: Want::Group("Game", "court"),
        answer: "shapes",
    },
    Step {
        title: "Give the court its size",
        note: &[
            "With court still in the name box, press add a value four times, then drag each number.",
            "The court wants a half width, a half height, a wall thickness and a color.",
        ],
        want: Want::Values("Game", "court", 3),
        answer: "shapes",
    },
    Step {
        title: "Make a group for the paddle",
        note: &[
            "Type paddle in the name box and press add a group, then give it a height and a y.",
        ],
        want: Want::Group("Game", "paddle"),
        answer: "shapes",
    },
    Step {
        title: "Add a system that builds the court",
        note: &[
            "Type setup.build in the name box and press add a system.",
            "A system is a rule. Its name says where it belongs and what it does.",
        ],
        want: Want::System("setup.build"),
        answer: "shapes",
    },
    Step {
        title: "Say that it runs once",
        note: &[
            "Pick setup.build, choose once in the port list and press the plus.",
            "A port says what a system reads, writes or does. Nothing says when it runs.",
        ],
        want: Want::Port("setup.build", "once"),
        answer: "shapes",
    },
    Step {
        title: "Let it make things",
        note: &[
            "Add the creates entities port, so the system is allowed to bring things into the world.",
        ],
        want: Want::Port("setup.build", "creates"),
        answer: "shapes",
    },
    Step {
        title: "Spawn the walls",
        note: &[
            "Choose spawn in the form list, press the plus inside setup.build, and set the parts.",
            "A wall is a Mesh, a Color, a Transform and a Scale. Three walls make the court.",
        ],
        want: Want::Spawns("setup.build", 3),
        answer: "shapes",
    },
    Step {
        title: "Spawn the paddle",
        note: &["One more spawn, with the paddle color and the paddle y."],
        want: Want::Spawns("setup.build", 4),
        answer: "shapes",
    },
    Step {
        title: "Add a Speed",
        note: &[
            "In Things, type Speed and press add a component.",
            "A component is one value that an entity can carry.",
        ],
        want: Want::Component("Speed"),
        answer: "steering",
    },
    Step {
        title: "Add a HalfExtent",
        note: &[
            "The paddle also carries how wide and how tall it is, so it can stay inside the walls.",
        ],
        want: Want::Component("HalfExtent"),
        answer: "steering",
    },
    Step {
        title: "Add a system that drives the paddle",
        note: &["Type paddle.drive in the name box and press add a system."],
        want: Want::System("paddle.drive"),
        answer: "steering",
    },
    Step {
        title: "Let it read the player",
        note: &[
            "Add a reads port and choose Input, then another for Time.",
            "Input.steering is minus one to one, and Time.dt is how long the turn took.",
        ],
        want: Want::Port("paddle.drive", "Input"),
        answer: "steering",
    },
    Step {
        title: "Move the paddle",
        note: &[
            "Add an every port over Speed, then a set statement that writes Transform.",
            "The paddle is anything that carries a Speed, so the rule never names one thing.",
        ],
        want: Want::Statements("paddle.drive", 1),
        answer: "steering",
    },
    Step {
        title: "Keep the round",
        note: &[
            "The ball waits until the player serves, so the game needs a shared value to remember that.",
            "Press show me the answer to read the finished ball, and keep going from there.",
        ],
        want: Want::Resource("Round"),
        answer: "bouncing",
    },
];
