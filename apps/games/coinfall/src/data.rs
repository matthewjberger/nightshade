pub struct Palette {
    pub top: [u32; 4],
    pub fill: u32,
    pub background: [u32; 2],
    pub walker: [u32; 2],
    pub tint: [f32; 4],
}

pub struct LevelSpec {
    pub name: &'static str,
    pub seconds: f32,
    pub palette: Palette,
    pub rows: &'static [&'static str],
}

pub const LEVELS: &[LevelSpec] = &[
    LevelSpec {
        name: "First Light",
        seconds: 34.0,
        palette: Palette {
            top: [20, 21, 22, 23],
            fill: 24,
            background: [1, 3],
            walker: [18, 19],
            tint: [1.00, 1.00, 1.00, 1.00],
        },
        rows: &[
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                   oo                                                                                   ",
            "                                   ==                                                                                   ",
            "                              ooo                                 ooo          ooo                                      ",
            "                  ooo         ===         oo              ==    ==   ==       ====              ooo                     ",
            "                                          ==                                 =                                          ",
            "   p    ooo T          b  w       T            b  w  oo w   g               =       T   w   b        T  w   oo g  F     ",
            "##################   ####################    ##################         ########################   #####################",
            "##################   ####################    ##################         ########################   #####################",
            "##################   ####################    ##################         ########################   #####################",
            "##################   ####################    ##################         ########################   #####################",
        ],
    },
    LevelSpec {
        name: "Sandline",
        seconds: 32.0,
        palette: Palette {
            top: [40, 41, 42, 43],
            fill: 24,
            background: [4, 5],
            walker: [15, 16],
            tint: [1.00, 1.00, 1.00, 1.00],
        },
        rows: &[
            "                                                                                                              ",
            "                                                                                                              ",
            "                                                                                                              ",
            "                                                                                                              ",
            "                                                                                                              ",
            "                                                                                                              ",
            "                                                                                                              ",
            "                                         ooo                                                                  ",
            "                         ooo             ===                ooo                 ooo                           ",
            "                oo       ===                      oo        ===          o      ===       oo                  ",
            "                ==                                ==                     =                ==                  ",
            "  p   ooo w c         k     w           c    w         k  w       w c               w k         c   oo  w F   ",
            "###############    ##############    ############    ##################     #############    #################",
            "###############    ##############    ############    ##################     #############    #################",
            "###############    ##############    ############    ##################     #############    #################",
            "###############    ##############    ############    ##################     #############    #################",
        ],
    },
    LevelSpec {
        name: "The Climb",
        seconds: 30.0,
        palette: Palette {
            top: [0, 1, 2, 3],
            fill: 4,
            background: [7, 6],
            walker: [11, 12],
            tint: [1.00, 1.00, 1.00, 1.00],
        },
        rows: &[
            "                                                                                                    ",
            "                                                                                                    ",
            "                                                                                                    ",
            "                                                                                                    ",
            "                                                                                                    ",
            "                                  ooo                                           ooo                 ",
            "                                  ===                        ooo                ===                 ",
            "                 oo           ooo                            ===            ooo                     ",
            "              oo ==   oo      ===      oo               ooo       oo        ===       ooo           ",
            "              ==      ==               ==     oo        ===       ==  oo              ===   oo      ",
            "            =                                 ==      =               ==                    ==      ",
            "  p ooo w =     T           w    b         w       w=     T      w            b      w    T    w F  ",
            "#####################    ####################    ####################    ###########################",
            "#####################    ####################    ####################    ###########################",
            "#####################    ####################    ####################    ###########################",
            "#####################    ####################    ####################    ###########################",
        ],
    },
    LevelSpec {
        name: "Coldstep",
        seconds: 28.0,
        palette: Palette {
            top: [80, 81, 82, 83],
            fill: 24,
            background: [0, 2],
            walker: [21, 22],
            tint: [1.00, 1.00, 1.00, 1.00],
        },
        rows: &[
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                                                                                                        ",
            "                                    ooo                       ooo                           ooo                         ",
            "                    ooo             ===           ooo         ===             ooo           ===             ooo         ",
            "              oo    ===       oo            oo    ===   oo              oo    ===     oo              oo    ===         ",
            "              ==              ==            ==          ==              ==            ==              ==                ",
            "  p ooo w S             w k           w S           w         n   w   S         w           k   w               w   F   ",
            "#############    ############    ##########    ########    ############    ##########    ############    ###############",
            "#############    ############    ##########    ########    ############    ##########    ############    ###############",
            "#############    ############    ##########    ########    ############    ##########    ############    ###############",
            "#############    ############    ##########    ########    ############    ##########    ############    ###############",
        ],
    },
    LevelSpec {
        name: "Last Call",
        seconds: 26.0,
        palette: Palette {
            top: [60, 61, 62, 63],
            fill: 4,
            background: [5, 4],
            walker: [24, 25],
            tint: [0.72, 0.60, 0.72, 1.00],
        },
        rows: &[
            "                                                                                                                                  ",
            "                                                                                                                                  ",
            "                                                                                                                                  ",
            "                                                                                                                                  ",
            "                                                                                                                                  ",
            "                                                                                                                                  ",
            "                                                                                                                                  ",
            "                               ooo                       ooo                         ooo                                          ",
            "                  ooo          ===          ooo          ===          ooo            ===          ooo               ooo           ",
            "            oo    ===     oo          oo    ===     oo          oo    ===       oo          oo    ===         oo    ===           ",
            "            ==            ==          ==            ==          ==              ==          ==                ==                  ",
            " p ooo  w        k  w            w m          w   k       w   m         w   k         w   m         w   w   k        m  w    w F  ",
            "###########    ##########    ########    ##########    ########    ############    ########    ##############    #################",
            "###########    ##########    ########    ##########    ########    ############    ########    ##############    #################",
            "###########    ##########    ########    ##########    ########    ############    ########    ##############    #################",
            "###########    ##########    ########    ##########    ########    ############    ########    ##############    #################",
        ],
    },
];

pub const GROUND: u8 = b'#';
pub const BLOCK: u8 = b'=';
pub const COIN: u8 = b'o';
pub const WALKER: u8 = b'w';
pub const START: u8 = b'p';
pub const FLAG: u8 = b'F';

pub const PROPS: [(u8, u32); 8] = [
    (b'T', 126),
    (b'c', 127),
    (b'b', 125),
    (b'g', 124),
    (b'm', 128),
    (b'n', 105),
    (b'k', 144),
    (b'S', 145),
];

pub const BACKGROUND_COLUMNS: u32 = 8;
pub const BACKGROUND_ROWS: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Steer,
    Jump,
    Confirm,
    Pause,
    Restart,
    StepLeft,
    StepRight,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Steer,
            Self::Jump,
            Self::Confirm,
            Self::Pause,
            Self::Restart,
            Self::StepLeft,
            Self::StepRight,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Steer => "steer",
            Self::Jump => "jump",
            Self::Confirm => "confirm",
            Self::Pause => "pause",
            Self::Restart => "restart",
            Self::StepLeft => "step left",
            Self::StepRight => "step right",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Press, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Steer => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::KeyD),
                    negative: Press::Key(KeyCode::KeyA),
                },
                Binding::Pair {
                    positive: Press::Key(KeyCode::ArrowRight),
                    negative: Press::Key(KeyCode::ArrowLeft),
                },
                Binding::Pair {
                    positive: Press::Pad(GamepadButton::DPadRight),
                    negative: Press::Pad(GamepadButton::DPadLeft),
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::KeyW),
                Binding::Key(KeyCode::ArrowUp),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Confirm => &[
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::Enter),
                Binding::Pad(GamepadButton::South),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::KeyP),
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Restart => &[
                Binding::Key(KeyCode::KeyR),
                Binding::Pad(GamepadButton::West),
            ],
            Self::StepLeft => &[
                Binding::Key(KeyCode::KeyA),
                Binding::Key(KeyCode::ArrowLeft),
                Binding::Pad(GamepadButton::DPadLeft),
            ],
            Self::StepRight => &[
                Binding::Key(KeyCode::KeyD),
                Binding::Key(KeyCode::ArrowRight),
                Binding::Pad(GamepadButton::DPadRight),
            ],
        }
    }
}
