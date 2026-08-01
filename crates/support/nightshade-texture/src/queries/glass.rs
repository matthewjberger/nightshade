use crate::data::Rgba;
use crate::queries::paint;

pub fn window(side: u32, frame: Rgba, glass: Rgba, light: Rgba) -> Vec<u8> {
    paint(side, |x, y| {
        let edge = x < 2 || y < 2 || x >= side - 2 || y >= side - 2;
        let bar = x == 7 || x == 8 || y == 7 || y == 8;
        if edge || bar {
            frame
        } else if x + y < 9 || (x + y > 20 && x > 9) {
            light
        } else {
            glass
        }
    })
}

pub fn pane(side: u32, frame: Rgba, glass: Rgba) -> Vec<u8> {
    paint(side, |x, y| {
        let edge = x < 2 || y < 2 || x >= side - 2 || y >= side - 2;
        if edge { frame } else { glass }
    })
}
