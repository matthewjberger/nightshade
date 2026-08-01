#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HumanoidSide {
    Center,
    Left,
    Right,
}

pub const HUMANOID_JOINTS: &[(&str, &[&str])] = &[
    ("hips", &["hips", "pelvis"]),
    ("spine", &["spine", "spine01"]),
    ("chest", &["spine1", "spine02"]),
    ("upperchest", &["spine2", "spine03"]),
    ("neck", &["neck", "neck01"]),
    ("head", &["head"]),
    ("headtip", &["headtopend", "headend"]),
    ("clavicle", &["shoulder", "clavicle"]),
    ("upperarm", &["arm", "upperarm"]),
    ("lowerarm", &["forearm", "lowerarm"]),
    ("hand", &["hand"]),
    ("thumb1", &["handthumb1", "thumb01"]),
    ("thumb2", &["handthumb2", "thumb02"]),
    ("thumb3", &["handthumb3", "thumb03"]),
    ("thumb4", &["handthumb4", "thumb04", "thumb04leaf"]),
    ("index1", &["handindex1", "index01"]),
    ("index2", &["handindex2", "index02"]),
    ("index3", &["handindex3", "index03"]),
    ("index4", &["handindex4", "index04", "index04leaf"]),
    ("middle1", &["handmiddle1", "middle01"]),
    ("middle2", &["handmiddle2", "middle02"]),
    ("middle3", &["handmiddle3", "middle03"]),
    ("middle4", &["handmiddle4", "middle04", "middle04leaf"]),
    ("ring1", &["handring1", "ring01"]),
    ("ring2", &["handring2", "ring02"]),
    ("ring3", &["handring3", "ring03"]),
    ("ring4", &["handring4", "ring04", "ring04leaf"]),
    ("pinky1", &["handpinky1", "pinky01", "littlefinger01"]),
    ("pinky2", &["handpinky2", "pinky02", "littlefinger02"]),
    ("pinky3", &["handpinky3", "pinky03", "littlefinger03"]),
    ("pinky4", &["handpinky4", "pinky04", "pinky04leaf"]),
    ("thigh", &["upleg", "thigh"]),
    ("calf", &["leg", "calf", "lowerleg"]),
    ("foot", &["foot"]),
    ("ball", &["toebase", "ball"]),
    ("balltip", &["toeend", "balltip"]),
];

const SIDE_PREFIXES: &[(&str, HumanoidSide)] = &[
    ("left", HumanoidSide::Left),
    ("right", HumanoidSide::Right),
    ("l_", HumanoidSide::Left),
    ("r_", HumanoidSide::Right),
];

const SIDE_SUFFIXES: &[(&str, HumanoidSide)] = &[
    ("_left", HumanoidSide::Left),
    ("_right", HumanoidSide::Right),
    ("_l", HumanoidSide::Left),
    ("_r", HumanoidSide::Right),
    (".l", HumanoidSide::Left),
    (".r", HumanoidSide::Right),
    ("left", HumanoidSide::Left),
    ("right", HumanoidSide::Right),
];

pub fn humanoid_joint(bone: &str) -> Option<(HumanoidSide, &'static str)> {
    let named = match bone.rsplit_once(':') {
        Some((_, suffix)) => suffix,
        None => bone,
    };
    let lowered = named.to_ascii_lowercase();
    let (side, rest) = split_side(&lowered);
    let core: String = rest
        .chars()
        .filter(|letter| letter.is_alphanumeric())
        .collect();
    if core.is_empty() {
        return None;
    }
    HUMANOID_JOINTS
        .iter()
        .find(|(_, aliases)| aliases.iter().any(|alias| *alias == core))
        .map(|(canonical, _)| (side, *canonical))
}

fn split_side(lowered: &str) -> (HumanoidSide, &str) {
    for (prefix, side) in SIDE_PREFIXES {
        if let Some(rest) = lowered.strip_prefix(prefix)
            && !rest.is_empty()
        {
            return (*side, rest);
        }
    }
    for (suffix, side) in SIDE_SUFFIXES {
        if let Some(rest) = lowered.strip_suffix(suffix)
            && !rest.is_empty()
        {
            return (*side, rest);
        }
    }
    (HumanoidSide::Center, lowered)
}
