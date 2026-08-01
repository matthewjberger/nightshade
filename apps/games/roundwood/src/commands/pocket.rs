use crate::data::{self, Item};
use crate::resources::{Pocket, Toast};

pub fn take_item(pocket: &mut Pocket, item: Item, count: u32) {
    pocket.counts[item.index()] += count;
}

pub fn sell_the_pocket(pocket: &mut Pocket) -> (u32, u32) {
    let mut goods = 0;
    let mut bells = 0;
    for item in Item::ALL {
        let held = pocket.counts[item.index()];
        goods += held;
        bells += held * item.price();
        pocket.counts[item.index()] = 0;
    }
    pocket.bells += bells;
    (goods, bells)
}

pub fn show_toast(toast: &mut Toast, line: impl Into<String>) {
    toast.line = line.into();
    toast.remaining = data::TOAST_TIME;
}
