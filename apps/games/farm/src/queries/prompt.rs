use crate::data::ShopKind;
use crate::queries::clock::{is_bed_hour, is_waking_hour};
use crate::resources::{Home, Prompt};
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};

pub fn repeats(prompt: &Prompt) -> bool {
    matches!(
        prompt,
        Prompt::Till(_, _)
            | Prompt::Water(_)
            | Prompt::Chop(_)
            | Prompt::Clear(_)
            | Prompt::Dig(_)
            | Prompt::Milk(_)
            | Prompt::Pick(_)
            | Prompt::Plant(_, _)
            | Prompt::Harvest(_)
    )
}

pub fn indoors(home: &Home, feet: Vec3, hour: f32) -> Prompt {
    let ground = Vec2::new(feet.x, feet.z);
    if let Some(bed) = home.bed
        && (bed - ground).magnitude() < theme::BED_RADIUS
        && is_bed_hour(hour)
    {
        return Prompt::Sleep;
    }
    if let Some(counter) = home.counter
        && (counter - ground).magnitude() < theme::COUNTER_RADIUS
        && is_waking_hour(hour)
        && let Some(kind) = shop_of(home)
    {
        return Prompt::EnterShop(kind);
    }
    if (home.exit - ground).magnitude() < theme::EXIT_RADIUS {
        return Prompt::LeaveHouse;
    }
    Prompt::None
}

fn shop_of(home: &Home) -> Option<ShopKind> {
    home.house.and_then(|kind| kind.house().shop)
}

pub fn text(prompt: &Prompt) -> Option<String> {
    match prompt {
        Prompt::None => None,
        Prompt::Talk(_, name) => Some(format!("Talk to {name}")),
        Prompt::Sleep => Some("Sleep until morning".to_string()),
        Prompt::EnterShop(kind) => Some(format!("Enter {}", kind.title())),
        Prompt::Till(_, _) => Some("Till the ground".to_string()),
        Prompt::Water(_) => Some("Water".to_string()),
        Prompt::Chop(_) => Some("Chop".to_string()),
        Prompt::Clear(_) => Some("Clear the stump".to_string()),
        Prompt::Dig(_) => Some("Dig up the rocks".to_string()),
        Prompt::Milk(_) => Some("Milk the cow".to_string()),
        Prompt::Cast => Some("Cast the line".to_string()),
        Prompt::Reel => Some("Reel in".to_string()),
        Prompt::Pull => Some("Pull!".to_string()),
        Prompt::Pick(_) => Some("Pick berries".to_string()),
        Prompt::EnterHollow => Some("Enter the Hollow".to_string()),
        Prompt::EnterHouse(kind) => Some(format!("Go into {}", kind.house().name)),
        Prompt::LeaveHouse => Some("Step outside".to_string()),
        Prompt::ClimbUp => Some("Climb up".to_string()),
        Prompt::ClimbDown => Some("Climb down".to_string()),
        Prompt::Open(_) => Some("Open the chest".to_string()),
        Prompt::Plant(_, kind) => Some(format!("Plant {}", kind.name())),
        Prompt::Harvest(_) => Some("Harvest".to_string()),
    }
}
