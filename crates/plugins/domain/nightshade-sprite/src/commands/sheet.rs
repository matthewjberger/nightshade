use crate::contract::TileTrim;

pub fn tile_trim(rgba: &[u8], width: u32, side: u32, columns: u32, table: &mut [TileTrim]) {
    let columns = columns.max(1);
    let side = side.max(1);
    for (index, slot) in table.iter_mut().enumerate() {
        let column = index as u32 % columns;
        let row = index as u32 / columns;
        let painted = |y: u32| {
            (0..side).any(|x| {
                let texel = ((row * side + y) * width + column * side + x) as usize;
                rgba.get(texel * 4 + 3).is_some_and(|alpha| *alpha > 0)
            })
        };
        let above = (0..side).take_while(|y| !painted(*y)).count() as u32;
        let below = (0..side).rev().take_while(|y| !painted(*y)).count() as u32;
        *slot = if above + below >= side {
            TileTrim::default()
        } else {
            TileTrim {
                above: above as u8,
                below: below as u8,
            }
        };
    }
}
