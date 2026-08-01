#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kit {
    Blocky,
    Proto,
}

pub struct ModelFile {
    pub name: &'static str,
    pub kit: Kit,
    pub bytes: &'static [u8],
}

pub fn kit_textures(kit: Kit) -> &'static [(&'static str, &'static [u8])] {
    match kit {
        Kit::Blocky => &[
            (
                "Textures/texture-a.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-a.png"),
            ),
            (
                "Textures/texture-b.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-b.png"),
            ),
            (
                "Textures/texture-c.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-c.png"),
            ),
            (
                "Textures/texture-d.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-d.png"),
            ),
            (
                "Textures/texture-e.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-e.png"),
            ),
            (
                "Textures/texture-f.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-f.png"),
            ),
            (
                "Textures/texture-g.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-g.png"),
            ),
            (
                "Textures/texture-h.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-h.png"),
            ),
            (
                "Textures/texture-i.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-i.png"),
            ),
            (
                "Textures/texture-j.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-j.png"),
            ),
            (
                "Textures/texture-k.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-k.png"),
            ),
            (
                "Textures/texture-l.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-l.png"),
            ),
            (
                "Textures/texture-m.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-m.png"),
            ),
            (
                "Textures/texture-n.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-n.png"),
            ),
            (
                "Textures/texture-o.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-o.png"),
            ),
            (
                "Textures/texture-p.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-p.png"),
            ),
            (
                "Textures/texture-q.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-q.png"),
            ),
            (
                "Textures/texture-r.png",
                include_bytes!("../../assets/models/blocky/Textures/texture-r.png"),
            ),
        ],
        Kit::Proto => &[(
            "Textures/colormap.png",
            include_bytes!("../../assets/models/proto/Textures/colormap.png"),
        )],
    }
}

pub const MODEL_COUNT: usize = 43;

pub static MODELS: [ModelFile; MODEL_COUNT] = [
    ModelFile {
        name: "blocky-character-a",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-a.glb"),
    },
    ModelFile {
        name: "blocky-character-b",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-b.glb"),
    },
    ModelFile {
        name: "blocky-character-c",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-c.glb"),
    },
    ModelFile {
        name: "blocky-character-d",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-d.glb"),
    },
    ModelFile {
        name: "blocky-character-e",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-e.glb"),
    },
    ModelFile {
        name: "blocky-character-f",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-f.glb"),
    },
    ModelFile {
        name: "blocky-character-g",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-g.glb"),
    },
    ModelFile {
        name: "blocky-character-h",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-h.glb"),
    },
    ModelFile {
        name: "blocky-character-i",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-i.glb"),
    },
    ModelFile {
        name: "blocky-character-j",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-j.glb"),
    },
    ModelFile {
        name: "blocky-character-k",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-k.glb"),
    },
    ModelFile {
        name: "blocky-character-l",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-l.glb"),
    },
    ModelFile {
        name: "blocky-character-m",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-m.glb"),
    },
    ModelFile {
        name: "blocky-character-n",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-n.glb"),
    },
    ModelFile {
        name: "blocky-character-o",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-o.glb"),
    },
    ModelFile {
        name: "blocky-character-p",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-p.glb"),
    },
    ModelFile {
        name: "blocky-character-q",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-q.glb"),
    },
    ModelFile {
        name: "blocky-character-r",
        kit: Kit::Blocky,
        bytes: include_bytes!("../../assets/models/blocky/blocky-character-r.glb"),
    },
    ModelFile {
        name: "animal-bison",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/animal-bison.glb"),
    },
    ModelFile {
        name: "animal-dog",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/animal-dog.glb"),
    },
    ModelFile {
        name: "animal-horse",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/animal-horse.glb"),
    },
    ModelFile {
        name: "button-floor-round",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/button-floor-round.glb"),
    },
    ModelFile {
        name: "column-rounded",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/column-rounded.glb"),
    },
    ModelFile {
        name: "column",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/column.glb"),
    },
    ModelFile {
        name: "crate-color",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/crate-color.glb"),
    },
    ModelFile {
        name: "crate",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/crate.glb"),
    },
    ModelFile {
        name: "flag",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/flag.glb"),
    },
    ModelFile {
        name: "floor-square",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/floor-square.glb"),
    },
    ModelFile {
        name: "ladder",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/ladder.glb"),
    },
    ModelFile {
        name: "pipe-corner",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/pipe-corner.glb"),
    },
    ModelFile {
        name: "pipe",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/pipe.glb"),
    },
    ModelFile {
        name: "shape-cube-half",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/shape-cube-half.glb"),
    },
    ModelFile {
        name: "shape-cube",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/shape-cube.glb"),
    },
    ModelFile {
        name: "shape-cylinder",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/shape-cylinder.glb"),
    },
    ModelFile {
        name: "shape-hexagon",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/shape-hexagon.glb"),
    },
    ModelFile {
        name: "shape-slope",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/shape-slope.glb"),
    },
    ModelFile {
        name: "stairs-narrow",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/stairs-narrow.glb"),
    },
    ModelFile {
        name: "stairs",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/stairs.glb"),
    },
    ModelFile {
        name: "target-a-round",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/target-a-round.glb"),
    },
    ModelFile {
        name: "wall-corner",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/wall-corner.glb"),
    },
    ModelFile {
        name: "wall-doorway",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/wall-doorway.glb"),
    },
    ModelFile {
        name: "wall-window-large",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/wall-window-large.glb"),
    },
    ModelFile {
        name: "wall",
        kit: Kit::Proto,
        bytes: include_bytes!("../../assets/models/proto/wall.glb"),
    },
];
