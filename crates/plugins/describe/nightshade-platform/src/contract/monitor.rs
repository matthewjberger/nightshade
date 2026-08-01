#[derive(Clone, Debug, Default)]
pub struct Monitor {
    pub index: usize,
    pub name: Option<String>,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub scale_factor: f32,
    pub refresh_rate_millihertz: Option<u32>,
    pub is_primary: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Monitors {
    pub monitors: Vec<Monitor>,
    pub primary_index: Option<usize>,
}

impl Monitor {
    pub fn min_x(&self) -> i32 {
        self.position.0
    }

    pub fn min_y(&self) -> i32 {
        self.position.1
    }

    pub fn max_x(&self) -> i32 {
        self.position.0 + self.size.0 as i32
    }

    pub fn max_y(&self) -> i32 {
        self.position.1 + self.size.1 as i32
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.min_x() && x < self.max_x() && y >= self.min_y() && y < self.max_y()
    }
}

impl Monitors {
    pub fn primary(&self) -> Option<&Monitor> {
        self.primary_index
            .and_then(|index| self.monitors.iter().find(|monitor| monitor.index == index))
    }

    pub fn at_index(&self, index: usize) -> Option<&Monitor> {
        self.monitors.iter().find(|monitor| monitor.index == index)
    }

    pub fn at_position(&self, x: i32, y: i32) -> Option<&Monitor> {
        self.monitors
            .iter()
            .find(|monitor| monitor.contains_point(x, y))
    }
}
