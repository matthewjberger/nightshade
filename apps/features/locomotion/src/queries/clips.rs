use nightshade_animation::contract::AnimationPlayer;

pub fn index_of(player: &AnimationPlayer, name: &str) -> Option<usize> {
    player
        .clips
        .iter()
        .position(|clip| clip.name.eq_ignore_ascii_case(name))
}

pub fn missing(player: &AnimationPlayer, names: &[&str]) -> Vec<String> {
    names
        .iter()
        .filter(|name| index_of(player, name).is_none())
        .map(|name| (*name).to_string())
        .collect()
}
