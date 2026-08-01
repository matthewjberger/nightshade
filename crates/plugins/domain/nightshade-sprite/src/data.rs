#[derive(Debug, Clone)]
struct Shelf {
    y: u32,
    height: u32,
    cursor_x: u32,
}

#[derive(Debug, Clone)]
pub struct ShelfPacker {
    size: u32,
    padding: u32,
    shelves: Vec<Shelf>,
}

impl ShelfPacker {
    pub fn new(size: u32) -> Self {
        Self {
            size,
            padding: 1,
            shelves: Vec::new(),
        }
    }

    pub fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if width == 0 || height == 0 || width > self.size || height > self.size {
            return None;
        }

        for shelf in &mut self.shelves {
            if shelf.height >= height
                && shelf.cursor_x + width <= self.size
                && shelf.height as f32 <= height as f32 * 1.5
            {
                let position = (shelf.cursor_x, shelf.y);
                shelf.cursor_x += width + self.padding;
                return Some(position);
            }
        }

        let next_shelf_y = self
            .shelves
            .last()
            .map(|shelf| shelf.y + shelf.height + self.padding)
            .unwrap_or(0);

        if next_shelf_y + height > self.size {
            return None;
        }

        self.shelves.push(Shelf {
            y: next_shelf_y,
            height,
            cursor_x: width + self.padding,
        });
        Some((0, next_shelf_y))
    }
}
