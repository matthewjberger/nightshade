pub const CATEGORY_COUNT: usize = 9;

pub const ALL_CATEGORIES: u32 = (1u32 << CATEGORY_COUNT) - 1;

pub struct Category {
    pub name: &'static str,
    pub blurb: &'static str,
}

pub const CATEGORIES: [Category; CATEGORY_COUNT] = [
    Category {
        name: "Settlements",
        blurb: "Villages, towns and cities with a well and a square.",
    },
    Category {
        name: "Ruins",
        blurb: "Broken towers, standing stones and old walls.",
    },
    Category {
        name: "Trees",
        blurb: "Oak in the warm lands, pine in the cold.",
    },
    Category {
        name: "Flowers",
        blurb: "Roses and dandelions across the meadows.",
    },
    Category {
        name: "Brush",
        blurb: "Dead bushes in the dry places.",
    },
    Category {
        name: "Rocks",
        blurb: "Boulders scattered over the hills.",
    },
    Category {
        name: "Hamlets",
        blurb: "Small clusters of homes and a farm.",
    },
    Category {
        name: "Landmarks",
        blurb: "Watchtowers, graveyards and lone camps.",
    },
    Category {
        name: "Ships",
        blurb: "Pirate ships adrift on the open sea.",
    },
];
