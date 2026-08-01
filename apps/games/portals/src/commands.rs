use crate::data;
use crate::queries::{body_center, body_reach, landing_is_clear, player_forward};
use crate::resources::{PlayerState, Solid};
use nightshade::prelude::{Vec3, Vec4};
use nightshade_portal::contract::PortalPlane;
use nightshade_portal::queries::portal_transform;

pub fn carry_through(
    player: &mut PlayerState,
    source: &PortalPlane,
    destination: &PortalPlane,
    solids: &[Solid],
) -> bool {
    let Some(carry) = portal_transform(source, destination) else {
        return false;
    };

    let center = body_center(player);
    let moved = carry * Vec4::new(center.x, center.y, center.z, 1.0);
    let turned = |vector: Vec3| {
        let out = carry * Vec4::new(vector.x, vector.y, vector.z, 0.0);
        Vec3::new(out.x, out.y, out.z)
    };

    let normal = destination.normal.normalize();
    let arrival = Vec3::new(moved.x, moved.y, moved.z);
    let stand = Vec3::new(0.0, data::PLAYER_HEIGHT * 0.5, 0.0);
    let reach = body_reach(normal) + data::TRAVEL_CLEARANCE;

    let depth = normal.dot(&(arrival - destination.center));
    let flat = arrival - normal * depth;

    let mut landing = None;
    for step in 0..data::TRAVEL_STEPS {
        let out = reach + step as f32 * data::TRAVEL_STEP;
        let candidate = flat + normal * out - stand;
        if landing_is_clear(candidate, solids) {
            landing = Some(candidate);
            break;
        }
    }
    let Some(landing) = landing else {
        return false;
    };

    player.position = landing;
    player.velocity = turned(player.velocity);

    let facing = turned(player_forward(player));
    let flat = Vec3::new(facing.x, 0.0, facing.z);
    let heading = if flat.norm() > data::TRAVEL_HEADING {
        Some(flat.normalize())
    } else {
        let out = Vec3::new(normal.x, 0.0, normal.z);
        (out.norm() > data::TRAVEL_HEADING).then(|| out.normalize())
    };
    if let Some(heading) = heading {
        player.yaw = (-heading.x).atan2(-heading.z);
    }
    true
}
