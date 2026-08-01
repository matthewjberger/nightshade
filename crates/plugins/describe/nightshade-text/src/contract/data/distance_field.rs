pub const SPREAD: u32 = 8;

pub const SUPERSAMPLE: u32 = 4;

pub struct DistanceCell {
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
    pub alphas: Vec<u8>,
}

pub fn distance_cell(
    coverage: &[u8],
    width: u32,
    height: u32,
    left: i32,
    top: i32,
) -> DistanceCell {
    let scale = SUPERSAMPLE as i32;
    let cell_left = left.div_euclid(scale) - SPREAD as i32;
    let cell_top = top.div_euclid(scale) + (top.rem_euclid(scale) != 0) as i32 + SPREAD as i32;
    let paste_x = (left - cell_left * scale) as usize;
    let paste_y = (cell_top * scale - top) as usize;
    let cell_width = (paste_x + width as usize).div_ceil(scale as usize) + SPREAD as usize;
    let cell_height = (paste_y + height as usize).div_ceil(scale as usize) + SPREAD as usize;
    let large_width = cell_width * scale as usize;
    let large_height = cell_height * scale as usize;

    let mut inside = vec![false; large_width * large_height];
    for row in 0..height as usize {
        for column in 0..width as usize {
            inside[(paste_y + row) * large_width + paste_x + column] =
                coverage[row * width as usize + column] >= 128;
        }
    }

    let to_inside = squared_distance_to(&inside, large_width, large_height, true);
    let to_outside = squared_distance_to(&inside, large_width, large_height, false);

    let reach = (SPREAD * SUPERSAMPLE) as f32;
    let samples = (scale * scale) as f32;
    let mut alphas = vec![0u8; cell_width * cell_height];
    for cell_row in 0..cell_height {
        for cell_column in 0..cell_width {
            let mut total = 0.0f32;
            for sub_row in 0..scale as usize {
                for sub_column in 0..scale as usize {
                    let index = (cell_row * scale as usize + sub_row) * large_width
                        + cell_column * scale as usize
                        + sub_column;
                    total += to_outside[index].sqrt() - to_inside[index].sqrt();
                }
            }
            let distance = total / samples;
            let mapped = 0.5 + distance / (2.0 * reach);
            alphas[cell_row * cell_width + cell_column] =
                (mapped.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }

    DistanceCell {
        width: cell_width as u32,
        height: cell_height as u32,
        left: cell_left,
        top: cell_top,
        alphas,
    }
}

fn squared_distance_to(inside: &[bool], width: usize, height: usize, target: bool) -> Vec<f32> {
    let far = (width * width + height * height) as f32;
    let mut field: Vec<f32> = inside
        .iter()
        .map(|&state| if state == target { 0.0 } else { far })
        .collect();

    let longest = width.max(height);
    let mut line = vec![0.0f32; longest];
    let mut result = vec![0.0f32; longest];
    let mut vertices = vec![0usize; longest];
    let mut boundaries = vec![0.0f32; longest + 1];

    for row in 0..height {
        line[..width].copy_from_slice(&field[row * width..(row + 1) * width]);
        lower_envelope(
            &line[..width],
            &mut result[..width],
            &mut vertices,
            &mut boundaries,
        );
        field[row * width..(row + 1) * width].copy_from_slice(&result[..width]);
    }
    for column in 0..width {
        for row in 0..height {
            line[row] = field[row * width + column];
        }
        lower_envelope(
            &line[..height],
            &mut result[..height],
            &mut vertices,
            &mut boundaries,
        );
        for row in 0..height {
            field[row * width + column] = result[row];
        }
    }
    field
}

fn lower_envelope(
    line: &[f32],
    result: &mut [f32],
    vertices: &mut [usize],
    boundaries: &mut [f32],
) {
    let crossing_of = |position: usize, vertex: usize| {
        ((line[position] + (position * position) as f32)
            - (line[vertex] + (vertex * vertex) as f32))
            / (2.0 * (position as f32 - vertex as f32))
    };
    let mut last = 0usize;
    vertices[0] = 0;
    boundaries[0] = f32::NEG_INFINITY;
    boundaries[1] = f32::INFINITY;
    for position in 1..line.len() {
        let mut crossing = crossing_of(position, vertices[last]);
        while crossing <= boundaries[last] {
            last -= 1;
            crossing = crossing_of(position, vertices[last]);
        }
        last += 1;
        vertices[last] = position;
        boundaries[last] = crossing;
        boundaries[last + 1] = f32::INFINITY;
    }

    let mut current = 0usize;
    for (position, slot) in result.iter_mut().enumerate() {
        while boundaries[current + 1] < position as f32 {
            current += 1;
        }
        let vertex = vertices[current];
        let offset = position as f32 - vertex as f32;
        *slot = offset * offset + line[vertex];
    }
}
