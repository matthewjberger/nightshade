use crate::data;

pub struct Block {
    pub name: &'static str,
    pub fields: &'static [&'static str],
}

pub struct Kind {
    pub name: &'static str,
    pub binding: &'static str,
}

pub struct Facility {
    pub name: &'static str,
    pub columns: &'static [(&'static str, &'static str)],
    pub bindings: &'static [&'static str],
    pub blocks: &'static [Block],
    pub kinds: &'static [Kind],
    pub sources: &'static [&'static str],
    pub functions: &'static [&'static str],
}

pub const FACILITIES: &[Facility] = &[
    PLACE, PAINT, SHAPE, SHEET, MARK, MANY, REACH, WALK, FIELD, DIG, NOISE, SPARK, SOLID, LOOK,
    SCREEN, SENSE, MATH,
];

const DIG: Facility = Facility {
    name: "dig",
    columns: &[(data::BLOCK_COLUMN, "text"), (data::DIG_COLUMN, "number")],
    bindings: &[],
    blocks: &[Block {
        name: data::LAND_RESOURCE,
        fields: &["seed", "hills", "shore", "reach"],
    }],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const FIELD: Facility = Facility {
    name: "field",
    columns: &[
        (data::FIELD_COLUMN, "text"),
        (data::CARVE_COLUMN, "number"),
        (data::BLEND_COLUMN, "number"),
    ],
    bindings: &[],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const WALK: Facility = Facility {
    name: "walk",
    columns: &[
        (data::GROUND_COLUMN, "number"),
        (data::PATH_COLUMN, "vector"),
        (data::PACE_COLUMN, "number"),
    ],
    bindings: &[],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const REACH: Facility = Facility {
    name: "reach",
    columns: &[
        (data::PICK_COLUMN, "number"),
        (data::PICKED_COLUMN, "number"),
    ],
    bindings: &[],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const MANY: Facility = Facility {
    name: "many",
    columns: &[
        (data::COPIES_COLUMN, "number"),
        (data::SPREAD_COLUMN, "vector"),
    ],
    bindings: &[],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const MARK: Facility = Facility {
    name: "mark",
    columns: &[],
    bindings: &[data::DECAL_BINDING],
    blocks: &[],
    kinds: &[Kind {
        name: "decal",
        binding: data::DECAL_BINDING,
    }],
    sources: &[],
    functions: &[],
};

const SHEET: Facility = Facility {
    name: "sheet",
    columns: &[(data::TILE_COLUMN, "number")],
    bindings: &[data::SPRITE_BINDING],
    blocks: &[],
    kinds: &[Kind {
        name: "sheet",
        binding: data::SPRITE_BINDING,
    }],
    sources: &[],
    functions: &[],
};

const PLACE: Facility = Facility {
    name: "place",
    columns: &[
        (data::TRANSFORM_COLUMN, "vector"),
        (data::SCALE_BINDING, "vector"),
        (data::ROTATION_COLUMN, "vector"),
        (data::PARENT_COLUMN, "thing"),
        (data::VISIBLE_COLUMN, "truth"),
        (data::LIFETIME_COLUMN, "number"),
    ],
    bindings: &[],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const PAINT: Facility = Facility {
    name: "paint",
    columns: &[
        (data::COLOR_BINDING, "vector"),
        (data::REPEAT_COLUMN, "number"),
    ],
    bindings: &[data::MESH_BINDING, data::TEXTURE_BINDING],
    blocks: &[],
    kinds: &[
        Kind {
            name: "mesh",
            binding: data::MESH_BINDING,
        },
        Kind {
            name: "texture",
            binding: data::TEXTURE_BINDING,
        },
    ],
    sources: &["builtin"],
    functions: &[],
};

const SHAPE: Facility = Facility {
    name: "shape",
    columns: &[],
    bindings: &[data::MODEL_BINDING],
    blocks: &[],
    kinds: &[Kind {
        name: "model",
        binding: data::MODEL_BINDING,
    }],
    sources: &["folder"],
    functions: &[],
};

const NOISE: Facility = Facility {
    name: "noise",
    columns: &[(data::VOLUME_COLUMN, "number")],
    bindings: &[data::SOUND_BINDING],
    blocks: &[],
    kinds: &[Kind {
        name: "sound",
        binding: data::SOUND_BINDING,
    }],
    sources: &[],
    functions: &[],
};

const SPARK: Facility = Facility {
    name: "spark",
    columns: &[],
    bindings: &[data::BURST_BINDING],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const SOLID: Facility = Facility {
    name: "solid",
    columns: &[(data::BODY_COLUMN, "text"), (data::BOUNCE_COLUMN, "number")],
    bindings: &[],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const LOOK: Facility = Facility {
    name: "look",
    columns: &[
        (data::LIGHT_COLUMN, "text"),
        (data::STRENGTH_COLUMN, "number"),
        (data::REACH_COLUMN, "number"),
    ],
    bindings: &[],
    blocks: &[
        Block {
            name: data::SUN_RESOURCE,
            fields: &["angle", "color", "strength", "shadows"],
        },
        Block {
            name: data::SKY_RESOURCE,
            fields: &["show", "ground", "look", "grid"],
        },
    ],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const SCREEN: Facility = Facility {
    name: "screen",
    columns: &[
        (data::LABEL_COLUMN, "text"),
        (data::SIZE_COLUMN, "number"),
        (data::ANCHOR_COLUMN, "vector"),
        (data::JUSTIFY_COLUMN, "text"),
        (data::PANEL_COLUMN, "vector"),
        (data::BUTTON_COLUMN, "number"),
        (data::CLICKED_COLUMN, "number"),
        (data::INK_COLUMN, "vector"),
    ],
    bindings: &[],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const SENSE: Facility = Facility {
    name: "sense",
    columns: &[],
    bindings: &[],
    blocks: &[
        Block {
            name: data::TIME_RESOURCE,
            fields: &["dt", "uptime", "longest"],
        },
        Block {
            name: data::INPUT_RESOURCE,
            fields: &[
                "steering", "forward", "serve", "pointer", "cursor", "press", "aim",
            ],
        },
        Block {
            name: data::WINDOW_RESOURCE,
            fields: &["title", "close", "rate", "gpu", "size"],
        },
        Block {
            name: data::CAMERA_RESOURCE,
            fields: &["eye", "at"],
        },
        Block {
            name: data::RANDOM_RESOURCE,
            fields: &["seed"],
        },
        Block {
            name: data::FLOW_RESOURCE,
            fields: &["scale"],
        },
        Block {
            name: data::SHAKE_RESOURCE,
            fields: &["x", "y"],
        },
        Block {
            name: data::COST_RESOURCE,
            fields: &["steps", "bytes", "commands"],
        },
    ],
    kinds: &[],
    sources: &[],
    functions: &[],
};

const MATH: Facility = Facility {
    name: "math",
    columns: &[],
    bindings: &[],
    blocks: &[],
    kinds: &[],
    sources: &[],
    functions: &[
        "abs",
        "sign",
        "min",
        "max",
        "clamp",
        "floor",
        "ceil",
        "sqrt",
        "sin",
        "cos",
        "length",
        "separation",
        "draw",
        "count",
        "remainder",
        "quotient",
        "bitAnd",
        "bitOr",
        "bitXor",
        "shift",
        "repeat",
    ],
};

pub fn form_of(name: &str) -> Option<crate::data::form::Form> {
    FACILITIES
        .iter()
        .flat_map(|facility| facility.columns.iter())
        .find(|(held, _)| *held == name)
        .and_then(|(_, form)| crate::data::form::form_of(form))
}

pub fn columns() -> impl Iterator<Item = &'static str> {
    FACILITIES
        .iter()
        .flat_map(|facility| facility.columns.iter().map(|(name, _)| *name))
}

pub fn bindings() -> impl Iterator<Item = &'static str> {
    FACILITIES
        .iter()
        .flat_map(|facility| facility.bindings.iter().copied())
}

pub fn blocks() -> impl Iterator<Item = &'static Block> {
    FACILITIES
        .iter()
        .flat_map(|facility| facility.blocks.iter())
}

pub fn kinds() -> impl Iterator<Item = &'static Kind> {
    FACILITIES.iter().flat_map(|facility| facility.kinds.iter())
}

pub fn sources() -> impl Iterator<Item = &'static str> {
    FACILITIES
        .iter()
        .flat_map(|facility| facility.sources.iter().copied())
}

pub fn functions() -> impl Iterator<Item = &'static str> {
    FACILITIES
        .iter()
        .flat_map(|facility| facility.functions.iter().copied())
}

pub fn holds_column(name: &str) -> bool {
    columns().any(|held| held == name)
}

pub fn holds_binding(name: &str) -> bool {
    bindings().any(|held| held == name)
}

pub fn holds_source(name: &str) -> bool {
    sources().any(|held| held == name)
}

pub fn binding_for(kind: &str) -> Option<&'static str> {
    kinds()
        .find(|held| held.name == kind)
        .map(|held| held.binding)
}

pub fn block(name: &str) -> Option<&'static Block> {
    blocks().find(|held| held.name == name)
}

pub fn code_of(function: &str) -> Option<usize> {
    functions().position(|held| held == function)
}
