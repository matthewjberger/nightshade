use nalgebra_glm::{Vec3, Vec4, vec3, vec4};

pub const MAX_LINES: u32 = 1_000_000;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Lines {
    pub lines: Vec<Line>,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(skip)]
    pub version: u64,
}

impl Lines {
    pub fn new(lines: Vec<Line>) -> Self {
        Self {
            lines,
            always_on_top: false,
            version: 0,
        }
    }

    pub fn new_always_on_top(lines: Vec<Line>) -> Self {
        Self {
            lines,
            always_on_top: true,
            version: 0,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    pub fn push(&mut self, line: Line) {
        self.lines.push(line);
        self.mark_dirty();
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.mark_dirty();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec4,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            start: vec3(0.0, 0.0, 0.0),
            end: vec3(1.0, 1.0, 1.0),
            color: vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}
