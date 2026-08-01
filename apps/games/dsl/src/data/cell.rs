#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    Nothing,
    Truth,
    Number,
    Vector,
    Handle,
    Text,
    List,
}

pub const KEPT: u32 = 1 << 31;

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub tag: Tag,
    pub width: u8,
    pub slot: u32,
    pub parts: [f32; 4],
}

impl Default for Cell {
    fn default() -> Self {
        Self::nothing()
    }
}

impl Cell {
    pub const fn nothing() -> Self {
        Self {
            tag: Tag::Nothing,
            width: 0,
            slot: 0,
            parts: [0.0; 4],
        }
    }

    pub const fn truth(value: bool) -> Self {
        Self {
            tag: Tag::Truth,
            width: 0,
            slot: 0,
            parts: [if value { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
        }
    }

    pub const fn number(value: f32) -> Self {
        Self {
            tag: Tag::Number,
            width: 0,
            slot: 0,
            parts: [value, 0.0, 0.0, 0.0],
        }
    }

    pub fn handle(seat: u32, life: u32) -> Self {
        Self {
            tag: Tag::Handle,
            width: 0,
            slot: seat,
            parts: [f32::from_bits(life), 0.0, 0.0, 0.0],
        }
    }

    pub fn life(&self) -> u32 {
        match self.tag {
            Tag::Handle => self.parts[0].to_bits(),
            _ => 0,
        }
    }

    pub const fn text(slot: u32) -> Self {
        Self {
            tag: Tag::Text,
            width: 0,
            slot,
            parts: [0.0; 4],
        }
    }

    pub const fn list(slot: u32) -> Self {
        Self {
            tag: Tag::List,
            width: 0,
            slot,
            parts: [0.0; 4],
        }
    }

    pub fn vector(parts: &[f32]) -> Self {
        let mut held = [0.0; 4];
        let width = parts.len().min(4);
        held[..width].copy_from_slice(&parts[..width]);
        Self {
            tag: Tag::Vector,
            width: width as u8,
            slot: 0,
            parts: held,
        }
    }

    pub fn value(&self) -> f32 {
        match self.tag {
            Tag::Number | Tag::Truth | Tag::Vector => self.parts[0],
            _ => 0.0,
        }
    }

    pub fn truthy(&self) -> bool {
        match self.tag {
            Tag::Truth | Tag::Number => self.parts[0] != 0.0,
            Tag::Nothing => false,
            _ => true,
        }
    }

    pub fn axis(&self, index: usize) -> f32 {
        match self.tag {
            Tag::Vector => self.parts.get(index).copied().unwrap_or_default(),
            Tag::Number | Tag::Truth if index == 0 => self.parts[0],
            _ => 0.0,
        }
    }

    pub fn row(&self) -> Option<u32> {
        match self.tag {
            Tag::Handle => Some(self.slot),
            _ => None,
        }
    }

    pub fn same(&self, other: &Cell) -> bool {
        match (self.tag, other.tag) {
            (Tag::Text, Tag::Text) => self.slot == other.slot,
            (Tag::Handle, Tag::Handle) => self.slot == other.slot && self.life() == other.life(),
            (Tag::Nothing, Tag::Nothing) => true,
            (Tag::Text, _) | (_, Tag::Text) => false,
            _ => (self.value() - other.value()).abs() < 1.0e-5,
        }
    }

    pub fn spread(&self, other: &Cell, join: fn(f32, f32) -> f32) -> Cell {
        if self.tag == Tag::Vector || other.tag == Tag::Vector {
            let width = self.width.max(other.width).max(1) as usize;
            let mut parts = [0.0; 4];
            for (index, part) in parts.iter_mut().enumerate().take(width) {
                let left = if self.tag == Tag::Vector {
                    self.parts[index]
                } else {
                    self.parts[0]
                };
                let right = if other.tag == Tag::Vector {
                    other.parts[index]
                } else {
                    other.parts[0]
                };
                *part = join(left, right);
            }
            return Cell {
                tag: Tag::Vector,
                width: width as u8,
                slot: 0,
                parts,
            };
        }
        Cell::number(join(self.value(), other.value()))
    }
}

pub fn axis_index(field: &str) -> usize {
    match field {
        "y" => 1,
        "z" => 2,
        "w" => 3,
        _ => 0,
    }
}
