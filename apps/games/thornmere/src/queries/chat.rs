use crate::data::CLASSES;
use crate::resources::Fill;

pub fn fill_text(template: &str, fill: &Fill) -> String {
    let class = &CLASSES[fill.speakers[0].class];
    template
        .replace("{a}", &fill.speakers[0].name)
        .replace("{b}", &fill.speakers[1].name)
        .replace("{c}", &fill.speakers[2].name)
        .replace("{class}", class.name)
        .replace("{role}", class.role)
        .replace("{monster}", &fill.monster)
        .replace("{place}", &fill.place)
        .replace("{item}", &fill.item)
        .replace("{price}", &fill.price)
        .replace("{quest}", &fill.quest)
        .replace("{guild}", &fill.guild)
        .replace("{player}", &fill.player)
        .replace("{level}", &fill.level)
        .replace("{killer}", &fill.killer)
}
