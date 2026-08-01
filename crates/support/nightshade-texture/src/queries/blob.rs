use crate::queries::paint;

pub fn radial_blob(side: u32, color: [u8; 3], reach: f32) -> Vec<u8> {
    let middle = (side as f32 - 1.0) / 2.0;
    paint(side, |x, y| {
        let distance = ((x as f32 - middle).powi(2) + (y as f32 - middle).powi(2)).sqrt();
        let alpha = ((1.0 - distance / middle) * reach).clamp(0.0, 1.0);
        [color[0], color[1], color[2], (alpha * 255.0) as u8]
    })
}
