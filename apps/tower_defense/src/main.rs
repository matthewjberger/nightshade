use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(TowerDefenseECS::new())
}

freecs::ecs! {
    GameWorld {
        entity_handle: EntityHandle => ENTITY_HANDLE,
        position: Position => POSITION,
        velocity: Velocity => VELOCITY,
        tower: Tower => TOWER,
        enemy: Enemy => ENEMY,
        projectile: Projectile => PROJECTILE,
        grid_cell: GridCell => GRID_CELL,
        health_bar: HealthBar => HEALTH_BAR,
        visual_effect: VisualEffect => VISUAL_EFFECT,
        range_indicator: RangeIndicator => RANGE_INDICATOR,
        money_popup: MoneyPopup => MONEY_POPUP,
    }
    GameResources {
        money: u32,
        lives: u32,
        wave: u32,
        game_state: GameState,
        selected_tower_type: TowerType,
        preview_entity: Option<Entity>,
        preview_range_lines: Vec<Entity>,
        ui_handles: UiHandles,
        spawn_timer: f32,
        enemies_to_spawn: Vec<EnemySpawnInfo>,
        wave_delay: f32,
        mouse_grid_pos: Option<(i32, i32)>,
        camera_entity: Entity,
        path: Vec<Vec3>,
        towers_by_position: HashMap<(i32, i32), freecs::Entity>,
        enemies_list: Vec<freecs::Entity>,
        projectiles_list: Vec<freecs::Entity>,
        effects_list: Vec<freecs::Entity>,
        hover_tower_text: Option<Entity>,
        grid_tiles: HashMap<(i32, i32), Entity>,
        tile_original_colors: HashMap<(i32, i32), Vec4>,
        last_hovered_tile: Option<(i32, i32)>,
        wave_announce_timer: f32,
        game_speed: f32,
        current_hp: u32,
        max_hp: u32,
        auto_start_waves: bool,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TowerType {
    #[default]
    Basic,
    Frost,
    Cannon,
    Sniper,
    Poison,
}

impl TowerType {
    fn cost(&self) -> u32 {
        match self {
            TowerType::Basic => 60,
            TowerType::Frost => 120,
            TowerType::Cannon => 200,
            TowerType::Sniper => 180,
            TowerType::Poison => 150,
        }
    }

    fn damage(&self) -> f32 {
        match self {
            TowerType::Basic => 15.0,
            TowerType::Frost => 8.0,
            TowerType::Cannon => 50.0,
            TowerType::Sniper => 80.0,
            TowerType::Poison => 5.0,
        }
    }

    fn range(&self) -> f32 {
        match self {
            TowerType::Basic => 3.0,
            TowerType::Frost => 2.5,
            TowerType::Cannon => 4.0,
            TowerType::Sniper => 6.0,
            TowerType::Poison => 2.8,
        }
    }

    fn fire_rate(&self) -> f32 {
        match self {
            TowerType::Basic => 0.5,
            TowerType::Frost => 1.0,
            TowerType::Cannon => 2.0,
            TowerType::Sniper => 3.0,
            TowerType::Poison => 0.8,
        }
    }

    fn color(&self) -> Vec4 {
        match self {
            TowerType::Basic => nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0),
            TowerType::Frost => nalgebra_glm::vec4(0.2, 0.6, 1.0, 1.0),
            TowerType::Cannon => nalgebra_glm::vec4(0.8, 0.2, 0.2, 1.0),
            TowerType::Sniper => nalgebra_glm::vec4(0.3, 0.3, 0.3, 1.0),
            TowerType::Poison => nalgebra_glm::vec4(0.6, 0.2, 0.8, 1.0),
        }
    }

    fn projectile_speed(&self) -> f32 {
        match self {
            TowerType::Basic => 12.0,
            TowerType::Frost => 8.0,
            TowerType::Cannon => 10.0,
            TowerType::Sniper => 20.0,
            TowerType::Poison => 10.0,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            TowerType::Basic => "Basic",
            TowerType::Frost => "Frost",
            TowerType::Cannon => "Cannon",
            TowerType::Sniper => "Sniper",
            TowerType::Poison => "Poison",
        }
    }

    fn all() -> [TowerType; 5] {
        [
            TowerType::Basic,
            TowerType::Frost,
            TowerType::Cannon,
            TowerType::Sniper,
            TowerType::Poison,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GameState {
    #[default]
    WaitingForWave,
    WaveInProgress,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EnemyType {
    #[default]
    Normal,
    Fast,
    Tank,
    Flying,
    Shielded,
    Healer,
    Boss,
}

impl EnemyType {
    fn base_health(&self) -> f32 {
        match self {
            EnemyType::Normal => 50.0,
            EnemyType::Fast => 30.0,
            EnemyType::Tank => 150.0,
            EnemyType::Flying => 40.0,
            EnemyType::Shielded => 60.0,
            EnemyType::Healer => 80.0,
            EnemyType::Boss => 500.0,
        }
    }

    fn health(&self, wave: u32) -> f32 {
        let health_multiplier = 1.0 + (wave as f32 - 1.0) * 0.5;
        self.base_health() * health_multiplier
    }

    fn speed(&self) -> f32 {
        match self {
            EnemyType::Normal => 2.0,
            EnemyType::Fast => 4.0,
            EnemyType::Tank => 1.0,
            EnemyType::Flying => 2.5,
            EnemyType::Shielded => 1.5,
            EnemyType::Healer => 1.8,
            EnemyType::Boss => 0.8,
        }
    }

    fn value(&self, wave: u32) -> u32 {
        let base = match self {
            EnemyType::Normal => 10,
            EnemyType::Fast => 15,
            EnemyType::Tank => 30,
            EnemyType::Flying => 20,
            EnemyType::Shielded => 25,
            EnemyType::Healer => 35,
            EnemyType::Boss => 100,
        };
        base + wave * 2
    }

    fn color(&self) -> Vec4 {
        match self {
            EnemyType::Normal => nalgebra_glm::vec4(0.8, 0.2, 0.2, 1.0),
            EnemyType::Fast => nalgebra_glm::vec4(1.0, 0.5, 0.0, 1.0),
            EnemyType::Tank => nalgebra_glm::vec4(0.4, 0.4, 0.4, 1.0),
            EnemyType::Flying => nalgebra_glm::vec4(0.5, 0.8, 1.0, 1.0),
            EnemyType::Shielded => nalgebra_glm::vec4(0.2, 0.6, 0.9, 1.0),
            EnemyType::Healer => nalgebra_glm::vec4(0.2, 0.9, 0.4, 1.0),
            EnemyType::Boss => nalgebra_glm::vec4(0.6, 0.0, 0.6, 1.0),
        }
    }

    fn scale(&self) -> Vec3 {
        match self {
            EnemyType::Normal => nalgebra_glm::vec3(0.4, 0.6, 0.4),
            EnemyType::Fast => nalgebra_glm::vec3(0.3, 0.5, 0.3),
            EnemyType::Tank => nalgebra_glm::vec3(0.6, 0.6, 0.6),
            EnemyType::Flying => nalgebra_glm::vec3(0.5, 0.5, 0.5),
            EnemyType::Shielded => nalgebra_glm::vec3(0.5, 0.5, 0.5),
            EnemyType::Healer => nalgebra_glm::vec3(0.4, 0.4, 0.4),
            EnemyType::Boss => nalgebra_glm::vec3(1.0, 1.0, 1.0),
        }
    }

    fn y_offset(&self) -> f32 {
        let base_offset = self.scale().y / 2.0 - 0.5;
        match self {
            EnemyType::Flying => 2.0 + base_offset,
            _ => base_offset,
        }
    }

    fn shield(&self) -> f32 {
        match self {
            EnemyType::Shielded => 30.0,
            EnemyType::Boss => 100.0,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnemySpawnInfo {
    enemy_type: EnemyType,
    spawn_time: f32,
}

#[derive(Default)]
struct UiHandles {
    money_text: Option<Entity>,
    lives_text: Option<Entity>,
    wave_text: Option<Entity>,
    hp_text: Option<Entity>,
    status_text: Option<Entity>,
    wave_announce_text: Option<Entity>,
    lives_bar: Option<Entity>,
    lives_bar_bg: Option<Entity>,
    tower_select_texts: Vec<Entity>,
}

struct TowerDefenseECS {
    game_world: GameWorld,
}

impl TowerDefenseECS {
    fn new() -> Self {
        Self {
            game_world: GameWorld::default(),
        }
    }

    fn spawn_tower(
        &mut self,
        world: &mut World,
        grid_x: i32,
        grid_z: i32,
        tower_type: TowerType,
    ) -> freecs::Entity {
        let position = nalgebra_glm::vec3(grid_x as f32, 0.0, grid_z as f32);
        let color = tower_type.color();

        let engine_entity = spawn_mesh(
            world,
            "Cylinder",
            position,
            nalgebra_glm::vec3(0.4, 0.8, 0.4),
        );

        let material_name = format!("Tower_{}", engine_entity.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: color.into(),
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(engine_entity, MaterialRef::new(material_name));

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | TOWER, 1)[0];
        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(position));
        self.game_world.set_tower(
            game_entity,
            Tower {
                tower_type,
                cooldown: 0.0,
                target: None,
                fire_animation: 0.0,
                tracking_time: 0.0,
                target_line: None,
            },
        );

        self.game_world.resources.money -= tower_type.cost();
        self.game_world
            .resources
            .towers_by_position
            .insert((grid_x, grid_z), game_entity);

        self.spawn_range_indicator(world, position, tower_type.range(), game_entity);

        game_entity
    }

    fn spawn_range_indicator(
        &mut self,
        world: &mut World,
        center: Vec3,
        range: f32,
        tower_entity: freecs::Entity,
    ) {
        let segments = 32;
        let mut line_entities = Vec::new();

        let tower_color = if let Some(tower) = self.game_world.get_tower(tower_entity) {
            tower.tower_type.color()
        } else {
            nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0)
        };

        for segment_index in 0..segments {
            let angle1 = (segment_index as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((segment_index + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let start =
                center + nalgebra_glm::vec3(angle1.cos() * range, 0.1, angle1.sin() * range);
            let end = center + nalgebra_glm::vec3(angle2.cos() * range, 0.1, angle2.sin() * range);

            let line_entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | LINES
                    | VISIBILITY,
                1,
            )[0];

            world.set_name(
                line_entity,
                Name(format!("Range Indicator Line {}", segment_index)),
            );
            world.set_local_transform(
                line_entity,
                LocalTransform {
                    translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                    ..Default::default()
                },
            );
            world.set_global_transform(line_entity, GlobalTransform::default());
            world.set_local_transform_dirty(line_entity, LocalTransformDirty);

            world.set_lines(
                line_entity,
                Lines::new(vec![Line {
                    start,
                    end,
                    color: nalgebra_glm::vec4(tower_color.x, tower_color.y, tower_color.z, 0.5),
                }]),
            );

            world.set_visibility(line_entity, Visibility { visible: false });

            line_entities.push(line_entity);
        }

        let game_entity = self.game_world.spawn_entities(RANGE_INDICATOR, 1)[0];
        self.game_world.set_range_indicator(
            game_entity,
            RangeIndicator {
                tower_entity,
                line_entities,
                visible: false,
            },
        );
    }

    fn initialize_grid(&mut self) {
        let grid_size = 12;
        for x in -grid_size / 2..=grid_size / 2 {
            for z in -grid_size / 2..=grid_size / 2 {
                let game_entity = self.game_world.spawn_entities(GRID_CELL, 1)[0];
                self.game_world.set_grid_cell(
                    game_entity,
                    GridCell {
                        x,
                        z,
                        occupied: false,
                        is_path: false,
                    },
                );
            }
        }
    }

    fn spawn_grid_tiles(&mut self, world: &mut World) {
        let grid_size = 12;
        for x in -grid_size / 2..=grid_size / 2 {
            for z in -grid_size / 2..=grid_size / 2 {
                let pos = nalgebra_glm::vec3(x as f32, 0.0, z as f32);
                let is_path = self.game_world.resources.path.windows(2).any(|w| {
                    let seg_start = w[0];
                    let seg_end = w[1];
                    let min_x = seg_start.x.min(seg_end.x);
                    let max_x = seg_start.x.max(seg_end.x);
                    let min_z = seg_start.z.min(seg_end.z);
                    let max_z = seg_start.z.max(seg_end.z);
                    pos.x >= min_x && pos.x <= max_x && pos.z >= min_z && pos.z <= max_z
                });

                let start_pos = self.game_world.resources.path[0];
                let start_half_size = 1.0;
                let is_start = (pos.x - start_pos.x).abs() <= start_half_size
                    && (pos.z - start_pos.z).abs() <= start_half_size;

                let end_pos = self.game_world.resources.path.last().unwrap();
                let end_half_size = 1.0;
                let is_end = (pos.x - end_pos.x).abs() <= end_half_size
                    && (pos.z - end_pos.z).abs() <= end_half_size;

                if is_start || is_end {
                    continue;
                }

                let color = if is_path {
                    nalgebra_glm::vec4(0.5, 0.3, 0.1, 1.0)
                } else {
                    nalgebra_glm::vec4(0.1, 0.3, 0.1, 1.0)
                };

                let tile_entity = spawn_mesh(
                    world,
                    "Cube",
                    nalgebra_glm::vec3(x as f32, -0.5, z as f32),
                    nalgebra_glm::vec3(0.9, 0.1, 0.9),
                );

                let material_name = format!("GridTile_{}", tile_entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: color.into(),
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(tile_entity, MaterialRef::new(material_name));

                self.game_world
                    .resources
                    .grid_tiles
                    .insert((x, z), tile_entity);
                self.game_world
                    .resources
                    .tile_original_colors
                    .insert((x, z), color);
            }
        }
    }

    fn create_path(&mut self, world: &mut World) {
        let path = vec![
            nalgebra_glm::vec3(-6.0, 0.0, 0.0),
            nalgebra_glm::vec3(-3.0, 0.0, 0.0),
            nalgebra_glm::vec3(-3.0, 0.0, -4.0),
            nalgebra_glm::vec3(3.0, 0.0, -4.0),
            nalgebra_glm::vec3(3.0, 0.0, 2.0),
            nalgebra_glm::vec3(-1.0, 0.0, 2.0),
            nalgebra_glm::vec3(-1.0, 0.0, 5.0),
            nalgebra_glm::vec3(6.0, 0.0, 5.0),
        ];

        self.game_world.resources.path = path.clone();

        for index in 0..path.len() - 1 {
            let start = path[index];
            let end = path[index + 1];
            let steps = 20;

            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let pos = start + (end - start) * t;
                let grid_x = pos.x.round() as i32;
                let grid_z = pos.z.round() as i32;

                let entities: Vec<_> = self.game_world.query_entities(GRID_CELL).collect();
                for entity in entities {
                    if let Some(cell) = self.game_world.get_grid_cell_mut(entity)
                        && cell.x == grid_x
                        && cell.z == grid_z
                    {
                        cell.is_path = true;
                        cell.occupied = true;
                    }
                }
            }
        }

        for index in 0..path.len() - 1 {
            let start = path[index];
            let end = path[index + 1];
            let steps = 10;

            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let pos = start + (end - start) * t;

                let is_at_start = (pos - path[0]).magnitude() < 0.5;
                let is_at_end = (pos - path[path.len() - 1]).magnitude() < 0.5;

                if is_at_start || is_at_end {
                    continue;
                }

                let path_segment = spawn_mesh(
                    world,
                    "Cube",
                    pos + nalgebra_glm::vec3(0.0, -0.5, 0.0),
                    nalgebra_glm::vec3(0.9, 0.1, 0.9),
                );

                let material_name = format!("PathSegment_{}", path_segment.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [0.5, 0.3, 0.1, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(path_segment, MaterialRef::new(material_name));
            }
        }

        let start_marker = spawn_mesh(
            world,
            "Cube",
            path[0] + nalgebra_glm::vec3(0.0, 0.0, 0.0),
            nalgebra_glm::vec3(1.5, 1.0, 1.5),
        );

        let material_name = format!("StartMarker_{}", start_marker.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [1.0, 0.5, 0.0, 1.0],
                emissive_factor: [0.5, 0.25, 0.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(start_marker, MaterialRef::new(material_name));

        let end_marker = spawn_mesh(
            world,
            "Cube",
            path[path.len() - 1] + nalgebra_glm::vec3(0.0, 0.25, 0.0),
            nalgebra_glm::vec3(2.0, 1.5, 2.0),
        );

        let material_name = format!("EndMarker_{}", end_marker.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [0.2, 0.2, 0.8, 1.0],
                emissive_factor: [0.1, 0.1, 0.4],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(end_marker, MaterialRef::new(material_name));
    }

    fn get_grid_position_from_mouse(&self, world: &World) -> Option<(i32, i32)> {
        let mouse = &world.resources.input.mouse;
        let mouse_pos = nalgebra_glm::vec2(mouse.position.x, mouse.position.y);

        if let Some(intersection) = get_ground_position_from_screen(world, mouse_pos, 0.0) {
            let grid_x = intersection.x.round() as i32;
            let grid_z = intersection.z.round() as i32;

            if (-10..=10).contains(&grid_x) && (-10..=10).contains(&grid_z) {
                return Some((grid_x, grid_z));
            }
        }

        None
    }

    fn can_place_tower_at(&self, x: i32, z: i32) -> bool {
        let pos = nalgebra_glm::vec3(x as f32, 0.0, z as f32);

        let start_pos = self.game_world.resources.path[0];
        let start_half_size = 1.0;
        let is_start = (pos.x - start_pos.x).abs() <= start_half_size
            && (pos.z - start_pos.z).abs() <= start_half_size;

        let end_pos = self.game_world.resources.path.last().unwrap();
        let end_half_size = 1.0;
        let is_end = (pos.x - end_pos.x).abs() <= end_half_size
            && (pos.z - end_pos.z).abs() <= end_half_size;

        if is_start || is_end {
            return false;
        }

        if self
            .game_world
            .resources
            .towers_by_position
            .contains_key(&(x, z))
        {
            return false;
        }

        for entity in self.game_world.query_entities(GRID_CELL) {
            if let Some(cell) = self.game_world.get_grid_cell(entity)
                && cell.x == x
                && cell.z == z
            {
                return !cell.occupied;
            }
        }
        false
    }

    fn mark_cell_occupied(&mut self, x: i32, z: i32) {
        let entities: Vec<_> = self.game_world.query_entities(GRID_CELL).collect();
        for entity in entities {
            if let Some(cell) = self.game_world.get_grid_cell_mut(entity)
                && cell.x == x
                && cell.z == z
            {
                cell.occupied = true;
            }
        }
    }

    fn despawn_entity(&mut self, world: &mut World, entity: freecs::Entity) {
        if let Some(handle) = self.game_world.get_entity_handle(entity) {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive { entity: handle.0 });
        }

        if let Some(tower) = self.game_world.get_tower(entity)
            && let Some(line_entity) = tower.target_line
        {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive {
                    entity: line_entity,
                });
        }

        let range_indicators_to_remove: Vec<_> = self
            .game_world
            .query_entities(RANGE_INDICATOR)
            .filter_map(|range_entity| {
                self.game_world
                    .get_range_indicator(range_entity)
                    .filter(|indicator| indicator.tower_entity == entity)
                    .map(|indicator| (range_entity, indicator.clone()))
            })
            .collect();

        for (range_entity, indicator) in range_indicators_to_remove {
            for &line in &indicator.line_entities {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive { entity: line });
            }
            self.game_world.despawn_entities(&[range_entity]);
        }

        self.game_world.despawn_entities(&[entity]);
    }

    fn sell_tower(
        &mut self,
        world: &mut World,
        tower_entity: freecs::Entity,
        grid_x: i32,
        grid_z: i32,
    ) {
        if let Some(tower) = self.game_world.get_tower(tower_entity) {
            let refund = (tower.tower_type.cost() as f32 * 0.7) as u32;
            self.game_world.resources.money += refund;

            let position = nalgebra_glm::vec3(grid_x as f32, 0.5, grid_z as f32);
            self.spawn_money_popup(world, position, refund as i32);

            let entities: Vec<_> = self.game_world.query_entities(GRID_CELL).collect();
            for entity in entities {
                if let Some(cell) = self.game_world.get_grid_cell_mut(entity)
                    && cell.x == grid_x
                    && cell.z == grid_z
                {
                    cell.occupied = false;
                    break;
                }
            }

            self.game_world
                .resources
                .towers_by_position
                .remove(&(grid_x, grid_z));

            let range_indicators_to_remove: Vec<_> = self
                .game_world
                .query_entities(RANGE_INDICATOR)
                .filter_map(|range_entity| {
                    self.game_world
                        .get_range_indicator(range_entity)
                        .filter(|indicator| indicator.tower_entity == tower_entity)
                        .map(|indicator| (range_entity, indicator.line_entities.clone()))
                })
                .collect();

            for (range_entity, line_entities) in range_indicators_to_remove {
                for line in line_entities {
                    world
                        .resources
                        .command_queue
                        .push(WorldCommand::DespawnRecursive { entity: line });
                }
                self.game_world.despawn_entities(&[range_entity]);
            }

            self.despawn_entity(world, tower_entity);
        }
    }

    fn spawn_money_popup(&mut self, world: &mut World, position: Vec3, amount: i32) {
        let (text, color) = if amount > 0 {
            (
                format!("+${}", amount),
                nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0),
            )
        } else {
            (
                format!("-${}", -amount),
                nalgebra_glm::vec4(1.0, 0.0, 0.0, 1.0),
            )
        };

        let text_index = world.resources.text_cache.add_text(&text);
        let text_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | TEXT | VISIBILITY,
            1,
        )[0];

        if let Some(name) = world.get_name_mut(text_entity) {
            *name = Name("Money Popup".to_string());
        }

        if let Some(transform) = world.get_local_transform_mut(text_entity) {
            transform.translation = position + nalgebra_glm::vec3(0.0, 1.5, 0.0);
        }

        if let Some(text_component) = world.get_text_mut(text_entity) {
            text_component.text_index = text_index;
            text_component.properties = TextProperties {
                font_size: 36.0,
                color,
                alignment: TextAlignment::Center,
                outline_width: 0.08,
                outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            };
            text_component.dirty = true;
        }

        let game_entity = self.game_world.spawn_entities(MONEY_POPUP, 1)[0];
        self.game_world.set_money_popup(
            game_entity,
            MoneyPopup {
                text_entity,
                lifetime: 0.0,
                amount,
            },
        );
    }

    fn update_money_popups(&mut self, world: &mut World, delta_time: f32) {
        let entities: Vec<_> = self.game_world.query_entities(MONEY_POPUP).collect();
        let mut popups_to_remove = Vec::new();

        for entity in entities {
            if let Some(mut popup) = self.game_world.get_money_popup(entity).copied() {
                popup.lifetime += delta_time;

                if popup.lifetime > 1.5 {
                    popups_to_remove.push((entity, popup.text_entity));
                    continue;
                }

                self.game_world.set_money_popup(entity, popup);

                if let Some(transform) = world.get_local_transform_mut(popup.text_entity) {
                    transform.translation.y += delta_time * 0.5;
                    world.set_local_transform_dirty(popup.text_entity, LocalTransformDirty);
                }

                if let Some(text_component) = world.get_text_mut(popup.text_entity) {
                    let alpha = (1.0 - (popup.lifetime / 1.5)).max(0.0);
                    text_component.properties.color.w = alpha;
                    text_component.dirty = true;
                }
            }
        }

        for (entity, text_entity) in popups_to_remove {
            if world.get_text(text_entity).is_some() {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive {
                        entity: text_entity,
                    });
            }
            self.game_world.despawn_entities(&[entity]);
        }
    }

    fn create_explosion_effect(&mut self, world: &mut World, position: Vec3) {
        for _ in 0..8 {
            let offset = nalgebra_glm::vec3(
                rand::rng().random_range(-0.5..0.5),
                rand::rng().random_range(0.0..0.5),
                rand::rng().random_range(-0.5..0.5),
            );

            let particle = spawn_mesh(
                world,
                "Sphere",
                position + offset,
                nalgebra_glm::vec3(0.15, 0.15, 0.15),
            );

            let material_name = format!("ExplosionParticle_{}", particle.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [1.0, 0.5, 0.0, 0.8],
                    emissive_factor: [1.0, 0.3, 0.0],
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(particle, MaterialRef::new(material_name));

            let game_entity = self
                .game_world
                .spawn_entities(ENTITY_HANDLE | VISUAL_EFFECT, 1)[0];
            self.game_world
                .set_entity_handle(game_entity, EntityHandle(particle));
            self.game_world.set_visual_effect(
                game_entity,
                VisualEffect {
                    effect_type: EffectType::Explosion,
                    lifetime: 0.5,
                    age: 0.0,
                },
            );
            self.game_world.resources.effects_list.push(game_entity);
        }
    }

    fn create_death_effect(&mut self, world: &mut World, position: Vec3) {
        let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
        let mut smoke_emitter = ParticleEmitter::smoke(position);
        smoke_emitter.one_shot = true;
        smoke_emitter.burst_count = 30;
        smoke_emitter.spawn_rate = 0.0;
        smoke_emitter.particle_lifetime_min = 0.8;
        smoke_emitter.particle_lifetime_max = 1.5;
        smoke_emitter.initial_velocity_min = 1.5;
        smoke_emitter.initial_velocity_max = 3.0;
        smoke_emitter.size_start = 0.3;
        smoke_emitter.size_end = 0.8;
        smoke_emitter.velocity_spread = 0.6;
        smoke_emitter.shape =
            nightshade::ecs::particles::components::EmitterShape::Sphere { radius: 0.2 };
        world.set_particle_emitter(entity, smoke_emitter);
    }

    fn create_poison_bubble_effect(&mut self, world: &mut World, position: Vec3) {
        let offset = nalgebra_glm::vec3(
            rand::rng().random_range(-0.2..0.2),
            rand::rng().random_range(0.0..0.5),
            rand::rng().random_range(-0.2..0.2),
        );

        let particle = spawn_mesh(
            world,
            "Sphere",
            position + offset,
            nalgebra_glm::vec3(0.08, 0.08, 0.08),
        );

        let material_name = format!("PoisonBubble_{}", particle.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [0.5, 0.0, 0.8, 0.6],
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(particle, MaterialRef::new(material_name));

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | VISUAL_EFFECT, 1)[0];
        self.game_world
            .set_entity_handle(game_entity, EntityHandle(particle));
        self.game_world.set_visual_effect(
            game_entity,
            VisualEffect {
                effect_type: EffectType::PoisonBubble,
                lifetime: 2.0,
                age: 0.0,
            },
        );
        self.game_world.resources.effects_list.push(game_entity);
    }

    fn create_muzzle_flash(&mut self, world: &mut World, position: Vec3) {
        let flash = spawn_mesh(world, "Sphere", position, nalgebra_glm::vec3(0.6, 0.6, 0.6));

        let material_name = format!("MuzzleFlash_{}", flash.id);
        material_registry_insert(
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [1.0, 1.0, 0.0, 1.0],
                emissive_factor: [2.0, 2.0, 0.0],
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(flash, MaterialRef::new(material_name));

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | VISUAL_EFFECT, 1)[0];
        self.game_world
            .set_entity_handle(game_entity, EntityHandle(flash));
        self.game_world.set_visual_effect(
            game_entity,
            VisualEffect {
                effect_type: EffectType::Explosion,
                lifetime: 0.15,
                age: 0.0,
            },
        );
        self.game_world.resources.effects_list.push(game_entity);

        for index in 0..8 {
            let angle = (index as f32 / 8.0) * std::f32::consts::TAU;
            let offset = nalgebra_glm::vec3(angle.cos() * 0.3, 0.0, angle.sin() * 0.3);

            let smoke = spawn_mesh(
                world,
                "Sphere",
                position + offset,
                nalgebra_glm::vec3(0.4, 0.4, 0.4),
            );

            let material_name = format!("MuzzleSmoke_{}", smoke.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.3, 0.3, 0.3, 0.8],
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(smoke, MaterialRef::new(material_name));

            let smoke_entity = self
                .game_world
                .spawn_entities(ENTITY_HANDLE | VISUAL_EFFECT, 1)[0];
            self.game_world
                .set_entity_handle(smoke_entity, EntityHandle(smoke));
            self.game_world.set_visual_effect(
                smoke_entity,
                VisualEffect {
                    effect_type: EffectType::Explosion,
                    lifetime: 2.0,
                    age: 0.0,
                },
            );
            self.game_world.resources.effects_list.push(smoke_entity);
        }
    }

    fn update_visual_effects(&mut self, world: &mut World, delta_time: f32) {
        let entities: Vec<_> = self.game_world.query_entities(VISUAL_EFFECT).collect();
        let mut effects_to_remove = Vec::new();

        for entity in entities {
            if let Some(mut effect) = self.game_world.get_visual_effect(entity).copied() {
                effect.age += delta_time;

                if effect.age >= effect.lifetime {
                    effects_to_remove.push(entity);
                    continue;
                }

                self.game_world.set_visual_effect(entity, effect);

                if let Some(handle) = self.game_world.get_entity_handle(entity) {
                    match effect.effect_type {
                        EffectType::Explosion => {
                            let progress = effect.age / effect.lifetime;
                            let scale_factor = 1.0 + progress * 2.0;

                            if let Some(transform) = world.get_local_transform_mut(handle.0) {
                                transform.scale = nalgebra_glm::vec3(
                                    0.15 * scale_factor,
                                    0.15 * scale_factor,
                                    0.15 * scale_factor,
                                );
                                world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                            }

                            if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                                && let Some(material) = registry_entry_by_name_mut(
                                    &mut world.resources.material_registry.registry,
                                    &material_ref.name,
                                )
                            {
                                let alpha = 1.0 - progress;
                                material.base_color[3] = alpha * 0.8;
                            }
                        }
                        EffectType::PoisonBubble => {
                            if let Some(transform) = world.get_local_transform_mut(handle.0) {
                                transform.translation.y += delta_time * 0.5;
                                world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                            }

                            if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                                && let Some(material) = registry_entry_by_name_mut(
                                    &mut world.resources.material_registry.registry,
                                    &material_ref.name,
                                )
                            {
                                let progress = effect.age / effect.lifetime;
                                let alpha = 1.0 - progress;
                                material.base_color[3] = alpha * 0.6;
                            }
                        }
                        EffectType::DeathParticle => {
                            if let Some(transform) = world.get_local_transform_mut(handle.0) {
                                transform.translation.y += delta_time * 1.0;
                                world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                            }

                            if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                                && let Some(material) = registry_entry_by_name_mut(
                                    &mut world.resources.material_registry.registry,
                                    &material_ref.name,
                                )
                            {
                                let progress = effect.age / effect.lifetime;
                                let alpha = 1.0 - progress;
                                material.base_color[3] = alpha * 0.8;
                            }
                        }
                    }
                }
            }
        }

        for entity in effects_to_remove {
            if let Some(idx) = self
                .game_world
                .resources
                .effects_list
                .iter()
                .position(|&e| e == entity)
            {
                self.game_world.resources.effects_list.remove(idx);
            }
            self.despawn_entity(world, entity);
        }
    }

    fn plan_wave(&mut self) {
        self.game_world.resources.wave += 1;
        let wave = self.game_world.resources.wave;
        let mut spawns = Vec::new();

        let is_boss_wave = wave.is_multiple_of(5);
        let enemy_count = 5 + wave * 2;

        let enemy_types = if is_boss_wave {
            match wave {
                5 => vec![
                    (EnemyType::Normal, 0.3),
                    (EnemyType::Fast, 0.3),
                    (EnemyType::Tank, 0.2),
                    (EnemyType::Boss, 0.2),
                ],
                10 => vec![
                    (EnemyType::Normal, 0.25),
                    (EnemyType::Fast, 0.25),
                    (EnemyType::Tank, 0.15),
                    (EnemyType::Flying, 0.15),
                    (EnemyType::Boss, 0.2),
                ],
                15 => vec![
                    (EnemyType::Normal, 0.2),
                    (EnemyType::Fast, 0.2),
                    (EnemyType::Tank, 0.15),
                    (EnemyType::Flying, 0.15),
                    (EnemyType::Shielded, 0.1),
                    (EnemyType::Boss, 0.2),
                ],
                _ => vec![
                    (EnemyType::Normal, 0.15),
                    (EnemyType::Fast, 0.15),
                    (EnemyType::Tank, 0.15),
                    (EnemyType::Flying, 0.15),
                    (EnemyType::Shielded, 0.1),
                    (EnemyType::Healer, 0.1),
                    (EnemyType::Boss, 0.2),
                ],
            }
        } else {
            match wave {
                1..=2 => vec![(EnemyType::Normal, 1.0)],
                3..=4 => vec![(EnemyType::Normal, 0.7), (EnemyType::Fast, 0.3)],
                6..=9 => vec![
                    (EnemyType::Normal, 0.4),
                    (EnemyType::Fast, 0.3),
                    (EnemyType::Tank, 0.2),
                    (EnemyType::Flying, 0.1),
                ],
                11..=14 => vec![
                    (EnemyType::Normal, 0.3),
                    (EnemyType::Fast, 0.25),
                    (EnemyType::Tank, 0.2),
                    (EnemyType::Flying, 0.15),
                    (EnemyType::Shielded, 0.1),
                ],
                _ => vec![
                    (EnemyType::Normal, 0.25),
                    (EnemyType::Fast, 0.2),
                    (EnemyType::Tank, 0.2),
                    (EnemyType::Flying, 0.15),
                    (EnemyType::Shielded, 0.1),
                    (EnemyType::Healer, 0.1),
                ],
            }
        };

        let spawn_interval = match wave {
            1..=3 => 1.0,
            4..=6 => 0.8,
            _ => 0.6,
        };

        let mut rng = rand::rng();
        let mut spawn_time = 0.0;

        for _ in 0..enemy_count {
            let roll: f32 = rng.random();
            let mut cumulative = 0.0;
            let mut selected_type = EnemyType::Normal;

            for (enemy_type, probability) in &enemy_types {
                cumulative += probability;
                if roll < cumulative {
                    selected_type = *enemy_type;
                    break;
                }
            }

            spawns.push(EnemySpawnInfo {
                enemy_type: selected_type,
                spawn_time,
            });
            spawn_time += spawn_interval;
        }

        self.game_world.resources.enemies_to_spawn = spawns;
        self.game_world.resources.spawn_timer = 0.0;
        self.game_world.resources.game_state = GameState::WaveInProgress;
        self.game_world.resources.wave_announce_timer = if is_boss_wave { 3.0 } else { 2.0 };
    }

    fn spawn_enemy(&mut self, world: &mut World, enemy_type: EnemyType) {
        if self.game_world.resources.path.is_empty() {
            return;
        }

        let position = self.game_world.resources.path[0];
        let wave = self.game_world.resources.wave;
        let base_y = enemy_type.y_offset();
        let color = enemy_type.color();

        let engine_entity = match enemy_type {
            EnemyType::Normal => {
                let main_entity = spawn_mesh(
                    world,
                    "Cube",
                    position + nalgebra_glm::vec3(0.0, base_y, 0.0),
                    nalgebra_glm::vec3(0.25, 0.3, 0.2),
                );
                let material_name = format!("GoblinBody_{}", main_entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: color.into(),
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(main_entity, MaterialRef::new(material_name));

                let head = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(head, Name("Goblin Head".to_string()));
                world.set_local_transform(
                    head,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.0, 0.25, 0.0),
                        scale: nalgebra_glm::vec3(0.15, 0.15, 0.15),
                        ..Default::default()
                    },
                );
                world.set_global_transform(head, GlobalTransform::default());
                world.set_local_transform_dirty(head, LocalTransformDirty);
                world.set_render_mesh(head, RenderMesh::new("Sphere"));
                let material_name = format!("GoblinHead_{}", head.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 0.9, color.y * 0.9, color.z * 0.9, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(head, MaterialRef::new(material_name));
                world.set_bounding_volume(head, BoundingVolume::from_mesh_type("Sphere"));
                world.set_parent(head, Parent(Some(main_entity)));

                let arm_left = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(arm_left, Name("Goblin Arm L".to_string()));
                world.set_local_transform(
                    arm_left,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(-0.2, -0.05, 0.0),
                        scale: nalgebra_glm::vec3(0.08, 0.2, 0.08),
                        ..Default::default()
                    },
                );
                world.set_global_transform(arm_left, GlobalTransform::default());
                world.set_local_transform_dirty(arm_left, LocalTransformDirty);
                world.set_render_mesh(arm_left, RenderMesh::new("Cube"));
                let material_name = format!("GoblinArmL_{}", arm_left.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(arm_left, MaterialRef::new(material_name));
                world.set_bounding_volume(arm_left, BoundingVolume::from_mesh_type("Cube"));
                world.set_parent(arm_left, Parent(Some(main_entity)));

                let arm_right = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(arm_right, Name("Goblin Arm R".to_string()));
                world.set_local_transform(
                    arm_right,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.2, -0.05, 0.0),
                        scale: nalgebra_glm::vec3(0.08, 0.2, 0.08),
                        ..Default::default()
                    },
                );
                world.set_global_transform(arm_right, GlobalTransform::default());
                world.set_local_transform_dirty(arm_right, LocalTransformDirty);
                world.set_render_mesh(arm_right, RenderMesh::new("Cube"));
                let material_name = format!("GoblinArmR_{}", arm_right.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(arm_right, MaterialRef::new(material_name));
                world.set_bounding_volume(arm_right, BoundingVolume::from_mesh_type("Cube"));
                world.set_parent(arm_right, Parent(Some(main_entity)));

                main_entity
            }
            EnemyType::Fast => {
                let main_entity = spawn_mesh(
                    world,
                    "Cone",
                    position + nalgebra_glm::vec3(0.0, base_y, 0.0),
                    nalgebra_glm::vec3(0.22, 0.4, 0.22),
                );
                let material_name = format!("FastBody_{}", main_entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: color.into(),
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(main_entity, MaterialRef::new(material_name));

                let head = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(head, Name("Fast Head".to_string()));
                world.set_local_transform(
                    head,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.0, 0.3, 0.0),
                        scale: nalgebra_glm::vec3(0.13, 0.13, 0.13),
                        ..Default::default()
                    },
                );
                world.set_global_transform(head, GlobalTransform::default());
                world.set_local_transform_dirty(head, LocalTransformDirty);
                world.set_render_mesh(head, RenderMesh::new("Sphere"));
                let material_name = format!("FastHead_{}", head.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(head, MaterialRef::new(material_name));
                world.set_bounding_volume(head, BoundingVolume::from_mesh_type("Sphere"));
                world.set_parent(head, Parent(Some(main_entity)));

                main_entity
            }
            EnemyType::Tank => {
                let main_entity = spawn_mesh(
                    world,
                    "Cube",
                    position + nalgebra_glm::vec3(0.0, base_y, 0.0),
                    nalgebra_glm::vec3(0.35, 0.4, 0.3),
                );
                let material_name = format!("TankBody_{}", main_entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: color.into(),
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(main_entity, MaterialRef::new(material_name));

                let head = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(head, Name("Tank Head".to_string()));
                world.set_local_transform(
                    head,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.0, 0.3, 0.0),
                        scale: nalgebra_glm::vec3(0.25, 0.25, 0.25),
                        ..Default::default()
                    },
                );
                world.set_global_transform(head, GlobalTransform::default());
                world.set_local_transform_dirty(head, LocalTransformDirty);
                world.set_render_mesh(head, RenderMesh::new("Cube"));
                let material_name = format!("TankHead_{}", head.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(head, MaterialRef::new(material_name));
                world.set_bounding_volume(head, BoundingVolume::from_mesh_type("Cube"));
                world.set_parent(head, Parent(Some(main_entity)));

                let shoulder_left = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(shoulder_left, Name("Tank Shoulder L".to_string()));
                world.set_local_transform(
                    shoulder_left,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(-0.25, 0.15, 0.0),
                        scale: nalgebra_glm::vec3(0.15, 0.15, 0.15),
                        ..Default::default()
                    },
                );
                world.set_global_transform(shoulder_left, GlobalTransform::default());
                world.set_local_transform_dirty(shoulder_left, LocalTransformDirty);
                world.set_render_mesh(shoulder_left, RenderMesh::new("Cube"));
                let material_name = format!("TankShoulderL_{}", shoulder_left.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(shoulder_left, MaterialRef::new(material_name));
                world.set_bounding_volume(shoulder_left, BoundingVolume::from_mesh_type("Cube"));
                world.set_parent(shoulder_left, Parent(Some(main_entity)));

                let shoulder_right = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(shoulder_right, Name("Tank Shoulder R".to_string()));
                world.set_local_transform(
                    shoulder_right,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.25, 0.15, 0.0),
                        scale: nalgebra_glm::vec3(0.15, 0.15, 0.15),
                        ..Default::default()
                    },
                );
                world.set_global_transform(shoulder_right, GlobalTransform::default());
                world.set_local_transform_dirty(shoulder_right, LocalTransformDirty);
                world.set_render_mesh(shoulder_right, RenderMesh::new("Cube"));
                let material_name = format!("TankShoulderR_{}", shoulder_right.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(shoulder_right, MaterialRef::new(material_name));
                world.set_bounding_volume(shoulder_right, BoundingVolume::from_mesh_type("Cube"));
                world.set_parent(shoulder_right, Parent(Some(main_entity)));

                main_entity
            }
            EnemyType::Flying => {
                let main_entity = spawn_mesh(
                    world,
                    "Sphere",
                    position + nalgebra_glm::vec3(0.0, base_y, 0.0),
                    nalgebra_glm::vec3(0.22, 0.28, 0.22),
                );
                let material_name = format!("FlyingBody_{}", main_entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: color.into(),
                        emissive_factor: [color.x * 0.35, color.y * 0.35, color.z * 0.35],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(main_entity, MaterialRef::new(material_name));

                let wing_left = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(wing_left, Name("Wing L".to_string()));
                world.set_local_transform(
                    wing_left,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(-0.3, 0.0, 0.0),
                        scale: nalgebra_glm::vec3(0.18, 0.1, 0.18),
                        rotation: nalgebra_glm::quat_angle_axis(
                            std::f32::consts::FRAC_PI_2,
                            &nalgebra_glm::vec3(0.0, 0.0, 1.0),
                        ),
                    },
                );
                world.set_global_transform(wing_left, GlobalTransform::default());
                world.set_local_transform_dirty(wing_left, LocalTransformDirty);
                world.set_render_mesh(wing_left, RenderMesh::new("Cone"));
                let material_name = format!("FlyingWingL_{}", wing_left.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.7],
                        alpha_mode: AlphaMode::Blend,
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(wing_left, MaterialRef::new(material_name));
                world.set_bounding_volume(wing_left, BoundingVolume::from_mesh_type("Cone"));
                world.set_parent(wing_left, Parent(Some(main_entity)));

                let wing_right = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(wing_right, Name("Wing R".to_string()));
                world.set_local_transform(
                    wing_right,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.3, 0.0, 0.0),
                        scale: nalgebra_glm::vec3(0.18, 0.1, 0.18),
                        rotation: nalgebra_glm::quat_angle_axis(
                            -std::f32::consts::FRAC_PI_2,
                            &nalgebra_glm::vec3(0.0, 0.0, 1.0),
                        ),
                    },
                );
                world.set_global_transform(wing_right, GlobalTransform::default());
                world.set_local_transform_dirty(wing_right, LocalTransformDirty);
                world.set_render_mesh(wing_right, RenderMesh::new("Cone"));
                let material_name = format!("FlyingWingR_{}", wing_right.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.7],
                        alpha_mode: AlphaMode::Blend,
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(wing_right, MaterialRef::new(material_name));
                world.set_bounding_volume(wing_right, BoundingVolume::from_mesh_type("Cone"));
                world.set_parent(wing_right, Parent(Some(main_entity)));

                main_entity
            }
            EnemyType::Shielded => {
                let main_entity = spawn_mesh(
                    world,
                    "Cube",
                    position + nalgebra_glm::vec3(0.0, base_y, 0.0),
                    nalgebra_glm::vec3(0.25, 0.3, 0.2),
                );
                let material_name = format!("ShieldedBody_{}", main_entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: color.into(),
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(main_entity, MaterialRef::new(material_name));

                let head = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(head, Name("Shielded Head".to_string()));
                world.set_local_transform(
                    head,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.0, 0.25, 0.0),
                        scale: nalgebra_glm::vec3(0.15, 0.15, 0.15),
                        ..Default::default()
                    },
                );
                world.set_global_transform(head, GlobalTransform::default());
                world.set_local_transform_dirty(head, LocalTransformDirty);
                world.set_render_mesh(head, RenderMesh::new("Sphere"));
                let material_name = format!("ShieldedHead_{}", head.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 0.9, color.y * 0.9, color.z * 0.9, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(head, MaterialRef::new(material_name));
                world.set_bounding_volume(head, BoundingVolume::from_mesh_type("Sphere"));
                world.set_parent(head, Parent(Some(main_entity)));

                let shield = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(shield, Name("Shield".to_string()));
                world.set_local_transform(
                    shield,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.2, 0.0, 0.0),
                        scale: nalgebra_glm::vec3(0.3, 0.06, 0.3),
                        ..Default::default()
                    },
                );
                world.set_global_transform(shield, GlobalTransform::default());
                world.set_local_transform_dirty(shield, LocalTransformDirty);
                world.set_render_mesh(shield, RenderMesh::new("Cylinder"));
                let material_name = format!("ShieldedShield_{}", shield.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [0.7, 0.7, 1.0, 0.65],
                        alpha_mode: AlphaMode::Blend,
                        emissive_factor: [0.3, 0.3, 0.6],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(shield, MaterialRef::new(material_name));
                world.set_bounding_volume(shield, BoundingVolume::from_mesh_type("Cylinder"));
                world.set_parent(shield, Parent(Some(main_entity)));

                main_entity
            }
            EnemyType::Healer => {
                let main_entity = spawn_mesh(
                    world,
                    "Torus",
                    position + nalgebra_glm::vec3(0.0, base_y, 0.0),
                    nalgebra_glm::vec3(0.25, 0.25, 0.25),
                );
                let material_name = format!("HealerBody_{}", main_entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: color.into(),
                        emissive_factor: [color.x * 0.5, color.y * 0.5, color.z * 0.5],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(main_entity, MaterialRef::new(material_name));

                let orb = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(orb, Name("Healer Orb".to_string()));
                world.set_local_transform(
                    orb,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                        scale: nalgebra_glm::vec3(0.16, 0.16, 0.16),
                        ..Default::default()
                    },
                );
                world.set_global_transform(orb, GlobalTransform::default());
                world.set_local_transform_dirty(orb, LocalTransformDirty);
                world.set_render_mesh(orb, RenderMesh::new("Sphere"));
                let material_name = format!("HealerOrb_{}", orb.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [0.3, 1.0, 0.3, 1.0],
                        emissive_factor: [0.0, 0.6, 0.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(orb, MaterialRef::new(material_name));
                world.set_bounding_volume(orb, BoundingVolume::from_mesh_type("Sphere"));
                world.set_parent(orb, Parent(Some(main_entity)));

                main_entity
            }
            EnemyType::Boss => {
                let main_entity = spawn_mesh(
                    world,
                    "Cube",
                    position + nalgebra_glm::vec3(0.0, base_y, 0.0),
                    nalgebra_glm::vec3(0.5, 0.6, 0.4),
                );
                let material_name = format!("BossBody_{}", main_entity.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: color.into(),
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(main_entity, MaterialRef::new(material_name));

                let head = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(head, Name("Boss Head".to_string()));
                world.set_local_transform(
                    head,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.0, 0.5, 0.0),
                        scale: nalgebra_glm::vec3(0.4, 0.4, 0.4),
                        ..Default::default()
                    },
                );
                world.set_global_transform(head, GlobalTransform::default());
                world.set_local_transform_dirty(head, LocalTransformDirty);
                world.set_render_mesh(head, RenderMesh::new("Cube"));
                let material_name = format!("BossHead_{}", head.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 0.7, color.y * 0.7, color.z * 0.7, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(head, MaterialRef::new(material_name));
                world.set_bounding_volume(head, BoundingVolume::from_mesh_type("Cube"));
                world.set_parent(head, Parent(Some(main_entity)));

                let crown = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(crown, Name("Boss Crown".to_string()));
                world.set_local_transform(
                    crown,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.0, 0.8, 0.0),
                        scale: nalgebra_glm::vec3(0.28, 0.28, 0.28),
                        ..Default::default()
                    },
                );
                world.set_global_transform(crown, GlobalTransform::default());
                world.set_local_transform_dirty(crown, LocalTransformDirty);
                world.set_render_mesh(crown, RenderMesh::new("Cone"));
                let material_name = format!("BossCrown_{}", crown.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [1.0, 0.85, 0.0, 1.0],
                        emissive_factor: [0.6, 0.4, 0.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(crown, MaterialRef::new(material_name));
                world.set_bounding_volume(crown, BoundingVolume::from_mesh_type("Cone"));
                world.set_parent(crown, Parent(Some(main_entity)));

                let shoulder_left = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(shoulder_left, Name("Boss Shoulder L".to_string()));
                world.set_local_transform(
                    shoulder_left,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(-0.4, 0.25, 0.0),
                        scale: nalgebra_glm::vec3(0.2, 0.25, 0.2),
                        ..Default::default()
                    },
                );
                world.set_global_transform(shoulder_left, GlobalTransform::default());
                world.set_local_transform_dirty(shoulder_left, LocalTransformDirty);
                world.set_render_mesh(shoulder_left, RenderMesh::new("Cube"));
                let material_name = format!("BossShoulderL_{}", shoulder_left.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 1.3, color.y * 1.3, color.z * 1.3, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(shoulder_left, MaterialRef::new(material_name));
                world.set_bounding_volume(shoulder_left, BoundingVolume::from_mesh_type("Cube"));
                world.set_parent(shoulder_left, Parent(Some(main_entity)));

                let shoulder_right = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | RENDER_MESH
                        | MATERIAL_REF
                        | BOUNDING_VOLUME
                        | PARENT,
                    1,
                )[0];
                world.set_name(shoulder_right, Name("Boss Shoulder R".to_string()));
                world.set_local_transform(
                    shoulder_right,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.4, 0.25, 0.0),
                        scale: nalgebra_glm::vec3(0.2, 0.25, 0.2),
                        ..Default::default()
                    },
                );
                world.set_global_transform(shoulder_right, GlobalTransform::default());
                world.set_local_transform_dirty(shoulder_right, LocalTransformDirty);
                world.set_render_mesh(shoulder_right, RenderMesh::new("Cube"));
                let material_name = format!("BossShoulderR_{}", shoulder_right.id);
                material_registry_insert(
                    &mut world.resources.material_registry,
                    material_name.clone(),
                    Material {
                        base_color: [color.x * 1.3, color.y * 1.3, color.z * 1.3, 1.0],
                        ..Default::default()
                    },
                );
                if let Some(&index) = world
                    .resources
                    .material_registry
                    .registry
                    .name_to_index
                    .get(&material_name)
                {
                    world
                        .resources
                        .material_registry
                        .registry
                        .add_reference(index);
                }
                world.set_material_ref(shoulder_right, MaterialRef::new(material_name));
                world.set_bounding_volume(shoulder_right, BoundingVolume::from_mesh_type("Cube"));
                world.set_parent(shoulder_right, Parent(Some(main_entity)));

                main_entity
            }
        };

        let game_entity = self
            .game_world
            .spawn_entities(ENTITY_HANDLE | POSITION | VELOCITY | ENEMY, 1)[0];

        self.game_world
            .set_entity_handle(game_entity, EntityHandle(engine_entity));
        self.game_world
            .set_position(game_entity, Position(position));
        self.game_world
            .set_velocity(game_entity, Velocity(nalgebra_glm::vec3(0.0, 0.0, 0.0)));

        self.game_world.set_enemy(
            game_entity,
            Enemy {
                health: enemy_type.health(wave),
                max_health: enemy_type.health(wave),
                shield_health: enemy_type.shield(),
                max_shield: enemy_type.shield(),
                speed: enemy_type.speed(),
                path_index: 0,
                path_progress: 0.0,
                value: enemy_type.value(wave),
                enemy_type,
                slow_duration: 0.0,
                poison_duration: 0.0,
                poison_damage: 0.0,
                is_flying: enemy_type == EnemyType::Flying,
            },
        );

        self.game_world.resources.enemies_list.push(game_entity);
    }

    fn input_system(&mut self, world: &mut World) {
        self.game_world.resources.mouse_grid_pos = self.get_grid_position_from_mouse(world);

        if let Some(prev_text) = self.game_world.resources.hover_tower_text.take() {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive { entity: prev_text });
        }

        if let Some((grid_x, grid_z)) = self.game_world.resources.mouse_grid_pos
            && let Some(&tower_entity) = self
                .game_world
                .resources
                .towers_by_position
                .get(&(grid_x, grid_z))
            && let Some(tower) = self.game_world.get_tower(tower_entity)
        {
            let sell_value = (tower.tower_type.cost() as f32 * 0.7) as u32;

            let text_index = world
                .resources
                .text_cache
                .add_text(format!("${}", sell_value));
            let text_entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | GLOBAL_TRANSFORM
                    | TEXT
                    | VISIBILITY,
                1,
            )[0];

            if let Some(name) = world.get_name_mut(text_entity) {
                *name = Name("Hover Tower Text".to_string());
            }

            if let Some(transform) = world.get_local_transform_mut(text_entity) {
                transform.translation = nalgebra_glm::vec3(grid_x as f32, 2.0, grid_z as f32);
            }

            if let Some(text_component) = world.get_text_mut(text_entity) {
                text_component.text_index = text_index;
                text_component.properties = TextProperties {
                    font_size: 36.0,
                    color: nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0),
                    alignment: TextAlignment::Center,
                    outline_width: 0.08,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.1,
                    ..Default::default()
                };
                text_component.dirty = true;
            }

            self.game_world.resources.hover_tower_text = Some(text_entity);
        }

        let left_clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::LEFT_CLICKED);
        let right_clicked = world
            .resources
            .input
            .mouse
            .state
            .contains(MouseState::RIGHT_CLICKED);

        let mut clicked_tower_hud = false;
        if left_clicked {
            let mouse_pos = world.resources.input.mouse.position;
            let hud_x_min = 10.0;
            let hud_x_max = 200.0;
            let hud_y_start = 10.0;
            let line_height = 28.0;
            let tower_count = TowerType::all().len();

            if mouse_pos.x >= hud_x_min && mouse_pos.x <= hud_x_max {
                for index in 0..tower_count {
                    let y_min = hud_y_start + (index as f32 * line_height);
                    let y_max = y_min + line_height;
                    if mouse_pos.y >= y_min && mouse_pos.y <= y_max {
                        let old_selection = self.game_world.resources.selected_tower_type;
                        self.game_world.resources.selected_tower_type = TowerType::all()[index];
                        if old_selection != self.game_world.resources.selected_tower_type {
                            self.update_tower_selection_hud(world);
                        }
                        clicked_tower_hud = true;
                        break;
                    }
                }
            }
        }

        if left_clicked
            && !clicked_tower_hud
            && let Some((grid_x, grid_z)) = self.game_world.resources.mouse_grid_pos
            && self.can_place_tower_at(grid_x, grid_z)
        {
            let tower_type = self.game_world.resources.selected_tower_type;
            if self.game_world.resources.money >= tower_type.cost() {
                let cost = tower_type.cost();
                self.spawn_tower(world, grid_x, grid_z, tower_type);
                self.mark_cell_occupied(grid_x, grid_z);
                self.spawn_money_popup(
                    world,
                    nalgebra_glm::vec3(grid_x as f32, 0.5, grid_z as f32),
                    -(cost as i32),
                );
            }
        }

        if right_clicked
            && let Some((grid_x, grid_z)) = self.game_world.resources.mouse_grid_pos
            && let Some(&tower_entity) = self
                .game_world
                .resources
                .towers_by_position
                .get(&(grid_x, grid_z))
        {
            self.sell_tower(world, tower_entity, grid_x, grid_z);
        }

        let key_1 = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Digit1);
        let key_2 = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Digit2);
        let key_3 = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Digit3);
        let key_4 = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Digit4);
        let key_5 = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Digit5);
        let key_bracket_left = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::BracketLeft);
        let key_bracket_right = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::BracketRight);
        let key_backslash = world
            .resources
            .input
            .keyboard
            .is_key_pressed(KeyCode::Backslash);

        let old_selection = self.game_world.resources.selected_tower_type;
        if key_1 {
            self.game_world.resources.selected_tower_type = TowerType::Basic;
        } else if key_2 {
            self.game_world.resources.selected_tower_type = TowerType::Frost;
        } else if key_3 {
            self.game_world.resources.selected_tower_type = TowerType::Cannon;
        } else if key_4 {
            self.game_world.resources.selected_tower_type = TowerType::Sniper;
        } else if key_5 {
            self.game_world.resources.selected_tower_type = TowerType::Poison;
        }

        if old_selection != self.game_world.resources.selected_tower_type {
            self.update_tower_selection_hud(world);
        }

        if key_bracket_left {
            self.game_world.resources.game_speed =
                (self.game_world.resources.game_speed - 0.5).max(0.5);
        } else if key_bracket_right {
            self.game_world.resources.game_speed =
                (self.game_world.resources.game_speed + 0.5).min(3.0);
        } else if key_backslash {
            self.game_world.resources.game_speed = 1.0;
        }
    }

    fn range_indicator_system(&mut self, world: &mut World) {
        let entities: Vec<_> = self.game_world.query_entities(RANGE_INDICATOR).collect();
        for entity in entities {
            if let Some(indicator) = self.game_world.get_range_indicator_mut(entity)
                && indicator.visible
            {
                indicator.visible = false;
                for &line_entity in &indicator.line_entities {
                    if let Some(visibility) = world.get_visibility_mut(line_entity) {
                        visibility.visible = false;
                    }
                }
            }
        }

        if let Some((grid_x, grid_z)) = self.game_world.resources.mouse_grid_pos
            && let Some(&tower_entity) = self
                .game_world
                .resources
                .towers_by_position
                .get(&(grid_x, grid_z))
        {
            let entities: Vec<_> = self.game_world.query_entities(RANGE_INDICATOR).collect();
            for entity in entities {
                if let Some(indicator) = self.game_world.get_range_indicator_mut(entity)
                    && indicator.tower_entity == tower_entity
                {
                    indicator.visible = true;
                    for &line_entity in &indicator.line_entities {
                        if let Some(visibility) = world.get_visibility_mut(line_entity) {
                            visibility.visible = true;
                        }
                    }
                    break;
                }
            }
        }
    }

    fn placement_preview_system(&mut self, world: &mut World) {
        if let Some(preview) = self.game_world.resources.preview_entity {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive { entity: preview });
            self.game_world.resources.preview_entity = None;
        }

        for entity in &self.game_world.resources.preview_range_lines {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive { entity: *entity });
        }
        self.game_world.resources.preview_range_lines.clear();

        if let Some((grid_x, grid_z)) = self.game_world.resources.mouse_grid_pos
            && self.can_place_tower_at(grid_x, grid_z)
        {
            let tower_type = self.game_world.resources.selected_tower_type;
            let can_afford = self.game_world.resources.money >= tower_type.cost();
            let line_alpha = if can_afford { 0.5 } else { 0.2 };

            let position = nalgebra_glm::vec3(grid_x as f32, 0.0, grid_z as f32);

            let preview = spawn_mesh(
                world,
                "Cylinder",
                position,
                nalgebra_glm::vec3(0.4, 0.8, 0.4),
            );

            let material_name = format!("Preview_{}", preview.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [
                        tower_type.color().x,
                        tower_type.color().y,
                        tower_type.color().z,
                        1.0,
                    ],
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(preview, MaterialRef::new(material_name));

            self.game_world.resources.preview_entity = Some(preview);

            let range = tower_type.range();
            let segments = 32;
            let tower_color = tower_type.color();

            for segment_index in 0..segments {
                let angle1 = (segment_index as f32 / segments as f32) * std::f32::consts::TAU;
                let angle2 = ((segment_index + 1) as f32 / segments as f32) * std::f32::consts::TAU;

                let start =
                    position + nalgebra_glm::vec3(angle1.cos() * range, 0.1, angle1.sin() * range);
                let end =
                    position + nalgebra_glm::vec3(angle2.cos() * range, 0.1, angle2.sin() * range);

                let line_entity = world.spawn_entities(
                    NAME | LOCAL_TRANSFORM
                        | GLOBAL_TRANSFORM
                        | LOCAL_TRANSFORM_DIRTY
                        | LINES
                        | VISIBILITY,
                    1,
                )[0];

                world.set_name(
                    line_entity,
                    Name(format!("Preview Range Line {}", segment_index)),
                );
                world.set_local_transform(
                    line_entity,
                    LocalTransform {
                        translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                        ..Default::default()
                    },
                );
                world.set_global_transform(line_entity, GlobalTransform::default());
                world.set_local_transform_dirty(line_entity, LocalTransformDirty);

                world.set_lines(
                    line_entity,
                    Lines::new(vec![Line {
                        start,
                        end,
                        color: nalgebra_glm::vec4(
                            tower_color.x,
                            tower_color.y,
                            tower_color.z,
                            line_alpha,
                        ),
                    }]),
                );

                world.set_visibility(line_entity, Visibility { visible: true });

                self.game_world
                    .resources
                    .preview_range_lines
                    .push(line_entity);
            }

            let box_size = 0.5;
            let box_height = 0.9;
            let y_base = -0.5;
            let y_top = y_base + box_height;

            let corners = [
                nalgebra_glm::vec3(box_size, 0.0, box_size),
                nalgebra_glm::vec3(-box_size, 0.0, box_size),
                nalgebra_glm::vec3(-box_size, 0.0, -box_size),
                nalgebra_glm::vec3(box_size, 0.0, -box_size),
            ];

            let mut box_lines = Vec::new();
            let box_color = if can_afford {
                nalgebra_glm::vec4(0.0, 1.0, 0.0, 0.8)
            } else {
                nalgebra_glm::vec4(1.0, 0.0, 0.0, 0.8)
            };

            for index in 0..4 {
                let next_index = (index + 1) % 4;
                box_lines.push(Line {
                    start: position + corners[index] + nalgebra_glm::vec3(0.0, y_base, 0.0),
                    end: position + corners[next_index] + nalgebra_glm::vec3(0.0, y_base, 0.0),
                    color: box_color,
                });
                box_lines.push(Line {
                    start: position + corners[index] + nalgebra_glm::vec3(0.0, y_top, 0.0),
                    end: position + corners[next_index] + nalgebra_glm::vec3(0.0, y_top, 0.0),
                    color: box_color,
                });
                box_lines.push(Line {
                    start: position + corners[index] + nalgebra_glm::vec3(0.0, y_base, 0.0),
                    end: position + corners[index] + nalgebra_glm::vec3(0.0, y_top, 0.0),
                    color: box_color,
                });
            }

            let mark_y = 0.1;
            let mark_color = if can_afford {
                nalgebra_glm::vec4(0.0, 1.0, 0.0, 0.9)
            } else {
                nalgebra_glm::vec4(1.0, 0.0, 0.0, 0.9)
            };

            if can_afford {
                let check_size = 0.2;
                let check_left = -0.15;
                let check_middle = -0.05;
                let check_right = 0.15;

                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(check_left, mark_y, box_size),
                    end: position + nalgebra_glm::vec3(check_middle, mark_y - check_size, box_size),
                    color: mark_color,
                });
                box_lines.push(Line {
                    start: position
                        + nalgebra_glm::vec3(check_middle, mark_y - check_size, box_size),
                    end: position
                        + nalgebra_glm::vec3(check_right, mark_y + check_size * 0.5, box_size),
                    color: mark_color,
                });

                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(check_left, mark_y, -box_size),
                    end: position
                        + nalgebra_glm::vec3(check_middle, mark_y - check_size, -box_size),
                    color: mark_color,
                });
                box_lines.push(Line {
                    start: position
                        + nalgebra_glm::vec3(check_middle, mark_y - check_size, -box_size),
                    end: position
                        + nalgebra_glm::vec3(check_right, mark_y + check_size * 0.5, -box_size),
                    color: mark_color,
                });

                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(box_size, mark_y, check_left),
                    end: position + nalgebra_glm::vec3(box_size, mark_y - check_size, check_middle),
                    color: mark_color,
                });
                box_lines.push(Line {
                    start: position
                        + nalgebra_glm::vec3(box_size, mark_y - check_size, check_middle),
                    end: position
                        + nalgebra_glm::vec3(box_size, mark_y + check_size * 0.5, check_right),
                    color: mark_color,
                });

                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(-box_size, mark_y, check_left),
                    end: position
                        + nalgebra_glm::vec3(-box_size, mark_y - check_size, check_middle),
                    color: mark_color,
                });
                box_lines.push(Line {
                    start: position
                        + nalgebra_glm::vec3(-box_size, mark_y - check_size, check_middle),
                    end: position
                        + nalgebra_glm::vec3(-box_size, mark_y + check_size * 0.5, check_right),
                    color: mark_color,
                });
            } else {
                let x_size = 0.25;

                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(-x_size, mark_y - x_size, box_size),
                    end: position + nalgebra_glm::vec3(x_size, mark_y + x_size, box_size),
                    color: mark_color,
                });
                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(x_size, mark_y - x_size, box_size),
                    end: position + nalgebra_glm::vec3(-x_size, mark_y + x_size, box_size),
                    color: mark_color,
                });

                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(-x_size, mark_y - x_size, -box_size),
                    end: position + nalgebra_glm::vec3(x_size, mark_y + x_size, -box_size),
                    color: mark_color,
                });
                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(x_size, mark_y - x_size, -box_size),
                    end: position + nalgebra_glm::vec3(-x_size, mark_y + x_size, -box_size),
                    color: mark_color,
                });

                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(box_size, mark_y - x_size, -x_size),
                    end: position + nalgebra_glm::vec3(box_size, mark_y + x_size, x_size),
                    color: mark_color,
                });
                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(box_size, mark_y - x_size, x_size),
                    end: position + nalgebra_glm::vec3(box_size, mark_y + x_size, -x_size),
                    color: mark_color,
                });

                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(-box_size, mark_y - x_size, -x_size),
                    end: position + nalgebra_glm::vec3(-box_size, mark_y + x_size, x_size),
                    color: mark_color,
                });
                box_lines.push(Line {
                    start: position + nalgebra_glm::vec3(-box_size, mark_y - x_size, x_size),
                    end: position + nalgebra_glm::vec3(-box_size, mark_y + x_size, -x_size),
                    color: mark_color,
                });
            }

            let box_entity = world.spawn_entities(
                NAME | LOCAL_TRANSFORM
                    | GLOBAL_TRANSFORM
                    | LOCAL_TRANSFORM_DIRTY
                    | LINES
                    | VISIBILITY,
                1,
            )[0];

            let box_name = if can_afford {
                "Affordable Box"
            } else {
                "Unaffordable Box"
            };
            world.set_name(box_entity, Name(box_name.to_string()));
            world.set_local_transform(
                box_entity,
                LocalTransform {
                    translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                    ..Default::default()
                },
            );
            world.set_global_transform(box_entity, GlobalTransform::default());
            world.set_local_transform_dirty(box_entity, LocalTransformDirty);
            world.set_lines(box_entity, Lines::new(box_lines));
            world.set_visibility(box_entity, Visibility { visible: true });

            self.game_world
                .resources
                .preview_range_lines
                .push(box_entity);
        }
    }

    fn update_lives_bar(&mut self, world: &mut World) {
        if let Some(bar_entity) = self.game_world.resources.ui_handles.lives_bar {
            let total_hp = (self.game_world.resources.lives - 1) * self.game_world.resources.max_hp
                + self.game_world.resources.current_hp;
            let max_total_hp = self.game_world.resources.lives * self.game_world.resources.max_hp;
            let health_percentage = total_hp as f32 / max_total_hp as f32;
            let bar_width = 5.8 * health_percentage;

            if let Some(transform) = world.get_local_transform_mut(bar_entity) {
                transform.translation = nalgebra_glm::vec3(-3.9 + bar_width / 2.0, 1.0, -7.9);
                transform.scale = nalgebra_glm::vec3(bar_width, 0.25, 0.1);
                world.set_local_transform_dirty(bar_entity, LocalTransformDirty);
            }

            if let Some(material_ref) = world.get_material_ref(bar_entity).cloned()
                && let Some(material) = registry_entry_by_name_mut(
                    &mut world.resources.material_registry.registry,
                    &material_ref.name,
                )
            {
                material.base_color = if health_percentage > 0.5 {
                    [0.0, 1.0, 0.0, 1.0]
                } else if health_percentage > 0.25 {
                    [1.0, 1.0, 0.0, 1.0]
                } else {
                    [1.0, 0.0, 0.0, 1.0]
                };
            }
        }
    }

    fn ui_update_system(&mut self, world: &mut World) {
        self.update_lives_bar(world);

        if let Some(money_entity) = self.game_world.resources.ui_handles.money_text
            && let Some(text) = world.get_text(money_entity)
        {
            world.resources.text_cache.set_text(
                text.text_index,
                format!("Money: ${}", self.game_world.resources.money),
            );
            if let Some(text_mut) = world.get_text_mut(money_entity) {
                text_mut.dirty = true;
            }
        }

        if let Some(lives_entity) = self.game_world.resources.ui_handles.lives_text
            && let Some(text) = world.get_text(lives_entity)
        {
            world.resources.text_cache.set_text(
                text.text_index,
                format!("Lives: {}", self.game_world.resources.lives),
            );
            if let Some(text_mut) = world.get_text_mut(lives_entity) {
                text_mut.dirty = true;
            }
        }

        if let Some(hp_entity) = self.game_world.resources.ui_handles.hp_text
            && let Some(text) = world.get_text(hp_entity)
        {
            world.resources.text_cache.set_text(
                text.text_index,
                format!(
                    "HP: {}/{}",
                    self.game_world.resources.current_hp, self.game_world.resources.max_hp
                ),
            );
            if let Some(text_mut) = world.get_text_mut(hp_entity) {
                text_mut.dirty = true;

                let hp_ratio = self.game_world.resources.current_hp as f32
                    / self.game_world.resources.max_hp as f32;
                text_mut.properties.color = if hp_ratio > 0.5 {
                    nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0)
                } else if hp_ratio > 0.25 {
                    nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
                } else {
                    nalgebra_glm::vec4(1.0, 0.0, 0.0, 1.0)
                };
            }
        }

        if let Some(wave_entity) = self.game_world.resources.ui_handles.wave_text
            && let Some(text) = world.get_text(wave_entity)
        {
            world.resources.text_cache.set_text(
                text.text_index,
                format!("Wave: {}", self.game_world.resources.wave),
            );
            if let Some(text_mut) = world.get_text_mut(wave_entity) {
                text_mut.dirty = true;
            }
        }

        if let Some(wave_announce_entity) = self.game_world.resources.ui_handles.wave_announce_text
        {
            if self.game_world.resources.wave_announce_timer > 0.0 {
                let delta_time =
                    world.resources.window.timing.delta_time * self.game_world.resources.game_speed;
                self.game_world.resources.wave_announce_timer -= delta_time;

                if let Some(text) = world.get_text(wave_announce_entity) {
                    let wave_num = self.game_world.resources.wave;
                    let is_boss_wave = wave_num.is_multiple_of(5);
                    let announce_text = if is_boss_wave {
                        format!("BOSS WAVE {}", wave_num)
                    } else {
                        format!("WAVE {}", wave_num)
                    };
                    world
                        .resources
                        .text_cache
                        .set_text(text.text_index, announce_text);
                    if let Some(text_mut) = world.get_text_mut(wave_announce_entity) {
                        text_mut.dirty = true;
                    }
                }

                if let Some(visibility) = world.get_visibility_mut(wave_announce_entity) {
                    visibility.visible = true;
                }
            } else if let Some(visibility) = world.get_visibility_mut(wave_announce_entity) {
                visibility.visible = false;
            }
        }
    }

    fn update_tower_selection_hud(&mut self, world: &mut World) {
        let selected = self.game_world.resources.selected_tower_type;
        for (index, tower_type) in TowerType::all().iter().enumerate() {
            if let Some(&entity) = self
                .game_world
                .resources
                .ui_handles
                .tower_select_texts
                .get(index)
            {
                let is_selected = *tower_type == selected;
                let base_color = tower_type.color();
                let text_color = if is_selected {
                    nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
                } else {
                    nalgebra_glm::vec4(base_color.x, base_color.y, base_color.z, 0.7)
                };

                if let Some(hud_text) = world.get_hud_text_mut(entity) {
                    hud_text.properties.color = text_color;
                    hud_text.dirty = true;
                }
            }
        }
    }

    fn tile_hover_system(&mut self, world: &mut World) {
        if let Some(last_pos) = self.game_world.resources.last_hovered_tile
            && let Some(&tile_entity) = self.game_world.resources.grid_tiles.get(&last_pos)
            && let Some(&original_color) = self
                .game_world
                .resources
                .tile_original_colors
                .get(&last_pos)
            && let Some(material_ref) = world.get_material_ref(tile_entity).cloned()
            && let Some(material) = registry_entry_by_name_mut(
                &mut world.resources.material_registry.registry,
                &material_ref.name,
            )
        {
            material.base_color = original_color.into();
        }

        if let Some(current_pos) = self.game_world.resources.mouse_grid_pos {
            let pos = nalgebra_glm::vec3(current_pos.0 as f32, 0.0, current_pos.1 as f32);

            let start_pos = self.game_world.resources.path[0];
            let start_half_size = 1.0;
            let is_start = (pos.x - start_pos.x).abs() <= start_half_size
                && (pos.z - start_pos.z).abs() <= start_half_size;

            let end_pos = self.game_world.resources.path.last().unwrap();
            let end_half_size = 1.0;
            let is_end = (pos.x - end_pos.x).abs() <= end_half_size
                && (pos.z - end_pos.z).abs() <= end_half_size;

            let is_path = self.game_world.resources.path.windows(2).any(|w| {
                let seg_start = w[0];
                let seg_end = w[1];
                let min_x = seg_start.x.min(seg_end.x);
                let max_x = seg_start.x.max(seg_end.x);
                let min_z = seg_start.z.min(seg_end.z);
                let max_z = seg_start.z.max(seg_end.z);
                pos.x >= min_x && pos.x <= max_x && pos.z >= min_z && pos.z <= max_z
            });

            if !is_start && !is_end && !is_path {
                if let Some(&tile_entity) = self.game_world.resources.grid_tiles.get(&current_pos)
                    && let Some(material_ref) = world.get_material_ref(tile_entity).cloned()
                    && let Some(material) = registry_entry_by_name_mut(
                        &mut world.resources.material_registry.registry,
                        &material_ref.name,
                    )
                {
                    material.base_color = [1.0, 1.0, 0.0, 1.0];
                }
                self.game_world.resources.last_hovered_tile = Some(current_pos);
            } else {
                self.game_world.resources.last_hovered_tile = None;
            }
        } else {
            self.game_world.resources.last_hovered_tile = None;
        }
    }

    fn wave_spawning_system(&mut self, world: &mut World) {
        let delta_time =
            world.resources.window.timing.delta_time * self.game_world.resources.game_speed;

        if self.game_world.resources.game_state == GameState::WaitingForWave {
            if self.game_world.resources.wave_delay > 0.0 {
                self.game_world.resources.wave_delay -= delta_time;
            } else {
                self.plan_wave();
            }
            return;
        }

        if self.game_world.resources.game_state != GameState::WaveInProgress {
            return;
        }

        self.game_world.resources.spawn_timer += delta_time;

        let current_time = self.game_world.resources.spawn_timer;
        let mut spawns_to_process = Vec::new();

        for (index, spawn_info) in self
            .game_world
            .resources
            .enemies_to_spawn
            .iter()
            .enumerate()
        {
            if spawn_info.spawn_time <= current_time {
                spawns_to_process.push((index, spawn_info.enemy_type));
            }
        }

        for (_index, enemy_type) in spawns_to_process.iter() {
            self.spawn_enemy(world, *enemy_type);
        }

        for &(index, _) in spawns_to_process.iter().rev() {
            self.game_world.resources.enemies_to_spawn.remove(index);
        }

        if self.game_world.resources.enemies_to_spawn.is_empty()
            && self.game_world.resources.enemies_list.is_empty()
        {
            self.game_world.resources.game_state = GameState::WaitingForWave;
            self.game_world.resources.wave_delay = 3.0;
        }
    }

    fn enemy_movement_system(&mut self, world: &mut World) {
        let delta_time =
            world.resources.window.timing.delta_time * self.game_world.resources.game_speed;
        let path = self.game_world.resources.path.clone();
        let mut enemies_to_remove = Vec::new();

        for entity in self.game_world.resources.enemies_list.clone() {
            if let Some(mut enemy) = self.game_world.get_enemy(entity).copied() {
                if enemy.path_index >= path.len() - 1 {
                    enemies_to_remove.push(entity);

                    if self.game_world.resources.current_hp > 0 {
                        self.game_world.resources.current_hp =
                            self.game_world.resources.current_hp.saturating_sub(1);
                    }

                    if self.game_world.resources.current_hp == 0 {
                        self.game_world.resources.current_hp = self.game_world.resources.max_hp;
                        self.game_world.resources.lives =
                            self.game_world.resources.lives.saturating_sub(1);

                        if self.game_world.resources.lives == 0 {
                            self.game_world.resources.game_state = GameState::GameOver;
                        }
                    }
                    continue;
                }

                let current_speed = if enemy.slow_duration > 0.0 {
                    enemy.speed * 0.5
                } else {
                    enemy.speed
                };

                enemy.slow_duration = (enemy.slow_duration - delta_time).max(0.0);

                if enemy.poison_duration > 0.0 {
                    enemy.health -= enemy.poison_damage * delta_time;
                    enemy.poison_duration = (enemy.poison_duration - delta_time).max(0.0);

                    if let Some(pos) = self.game_world.get_position(entity) {
                        self.create_poison_bubble_effect(
                            world,
                            pos.0 + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0),
                        );
                    }

                    if enemy.health <= 0.0 {
                        self.game_world.resources.money += enemy.value;
                        let death_pos = if let Some(pos) = self.game_world.get_position(entity) {
                            pos.0 + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0)
                        } else {
                            nalgebra_glm::vec3(0.0, 0.0, 0.0)
                        };

                        self.spawn_money_popup(world, death_pos, enemy.value as i32);
                        self.create_death_effect(world, death_pos);

                        if let Some(idx) = self
                            .game_world
                            .resources
                            .enemies_list
                            .iter()
                            .position(|&e| e == entity)
                        {
                            self.game_world.resources.enemies_list.remove(idx);
                        }
                        enemies_to_remove.push(entity);
                        continue;
                    }
                }

                let start = path[enemy.path_index];
                let end = path[enemy.path_index + 1];
                let segment_length = (end - start).magnitude();

                enemy.path_progress += (current_speed * delta_time) / segment_length;

                while enemy.path_progress >= 1.0 && enemy.path_index < path.len() - 1 {
                    enemy.path_progress -= 1.0;
                    enemy.path_index += 1;

                    if enemy.path_index >= path.len() - 1 {
                        enemies_to_remove.push(entity);

                        if self.game_world.resources.current_hp > 0 {
                            self.game_world.resources.current_hp =
                                self.game_world.resources.current_hp.saturating_sub(1);
                        }

                        if self.game_world.resources.current_hp == 0 {
                            self.game_world.resources.current_hp = self.game_world.resources.max_hp;
                            self.game_world.resources.lives =
                                self.game_world.resources.lives.saturating_sub(1);

                            if self.game_world.resources.lives == 0 {
                                self.game_world.resources.game_state = GameState::GameOver;
                            }
                        }
                        break;
                    }
                }

                if enemy.path_index < path.len() - 1 && !enemies_to_remove.contains(&entity) {
                    let current_start = path[enemy.path_index];
                    let current_end = path[enemy.path_index + 1];
                    let new_position =
                        current_start + (current_end - current_start) * enemy.path_progress;

                    self.game_world.set_enemy(entity, enemy);
                    self.game_world.set_position(entity, Position(new_position));

                    if let Some(handle) = self.game_world.get_entity_handle(entity)
                        && let Some(transform) = world.get_local_transform_mut(handle.0)
                    {
                        let visual_position = new_position
                            + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0);
                        transform.translation = visual_position;
                        world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                    }
                }
            }
        }

        for entity in enemies_to_remove {
            if let Some(idx) = self
                .game_world
                .resources
                .enemies_list
                .iter()
                .position(|&e| e == entity)
            {
                self.game_world.resources.enemies_list.remove(idx);
            }
            self.despawn_entity(world, entity);
        }
    }

    fn tower_targeting_system(&mut self, world: &mut World) {
        let delta_time =
            world.resources.window.timing.delta_time * self.game_world.resources.game_speed;
        let tower_entities: Vec<_> = self.game_world.query_entities(TOWER).collect();

        for tower_entity in tower_entities {
            if let Some(mut tower) = self.game_world.get_tower(tower_entity).copied() {
                let is_sniper = tower.tower_type == TowerType::Sniper;

                if is_sniper {
                    let mut target_valid = false;
                    if let Some(current_target) = tower.target
                        && self
                            .game_world
                            .resources
                            .enemies_list
                            .contains(&current_target)
                    {
                        target_valid = true;
                    }

                    if !target_valid {
                        tower.target = None;
                        tower.tracking_time = 0.0;

                        if let Some(line_entity) = tower.target_line.take() {
                            world
                                .resources
                                .command_queue
                                .push(WorldCommand::DespawnRecursive {
                                    entity: line_entity,
                                });
                        }
                    }
                } else if let Some(target) = tower.target
                    && !self.game_world.resources.enemies_list.contains(&target)
                {
                    tower.target = None;
                }

                if tower.target.is_none()
                    && let Some(tower_pos) = self.game_world.get_position(tower_entity).copied()
                {
                    let range = tower.tower_type.range();
                    let mut closest_enemy: Option<(freecs::Entity, f32)> = None;
                    let enemies_list = self.game_world.resources.enemies_list.clone();

                    for &enemy_entity in &enemies_list {
                        if let Some(enemy_pos) = self.game_world.get_position(enemy_entity).copied()
                        {
                            let distance = (enemy_pos.0 - tower_pos.0).magnitude();
                            if distance <= range {
                                if let Some((_, closest_dist)) = closest_enemy {
                                    if distance < closest_dist {
                                        closest_enemy = Some((enemy_entity, distance));
                                    }
                                } else {
                                    closest_enemy = Some((enemy_entity, distance));
                                }
                            }
                        }
                    }

                    if let Some((enemy, _)) = closest_enemy {
                        tower.target = Some(enemy);
                    }
                } else if is_sniper && tower.target.is_some() {
                    tower.tracking_time += delta_time;

                    if let (Some(target), Some(tower_pos)) = (
                        tower.target,
                        self.game_world.get_position(tower_entity).copied(),
                    ) {
                        for &enemy_entity in &self.game_world.resources.enemies_list.clone() {
                            if enemy_entity == target {
                                if let Some(enemy_pos) =
                                    self.game_world.get_position(enemy_entity).copied()
                                {
                                    if tower.target_line.is_none() {
                                        let line_entity = world.spawn_entities(
                                            LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LINES | VISIBILITY,
                                            1,
                                        )[0];
                                        tower.target_line = Some(line_entity);
                                    }

                                    if let Some(line_entity) = tower.target_line {
                                        if let Some(lines_comp) = world.get_lines_mut(line_entity) {
                                            lines_comp.lines.clear();
                                            lines_comp.lines.push(Line {
                                                start: tower_pos.0
                                                    + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                                                end: enemy_pos.0
                                                    + nalgebra_glm::vec3(0.0, 0.3, 0.0),
                                                color: nalgebra_glm::vec4(1.0, 0.0, 0.0, 0.8),
                                            });
                                        }

                                        if let Some(visibility) =
                                            world.get_visibility_mut(line_entity)
                                        {
                                            visibility.visible = true;
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }

                self.game_world.set_tower(tower_entity, tower);
            }
        }
    }

    fn tower_shooting_system(&mut self, world: &mut World) {
        let delta_time =
            world.resources.window.timing.delta_time * self.game_world.resources.game_speed;
        let tower_entities: Vec<_> = self.game_world.query_entities(TOWER).collect();

        for tower_entity in tower_entities {
            if let Some(mut tower) = self.game_world.get_tower(tower_entity).copied() {
                tower.cooldown = (tower.cooldown - delta_time).max(0.0);

                let mut needs_fire_animation_update = false;
                let mut animation_scale = 1.0;

                if tower.fire_animation > 0.0 {
                    tower.fire_animation -= delta_time * 3.0;
                    needs_fire_animation_update = true;
                    animation_scale = 1.0 + tower.fire_animation * 0.2;
                }

                if tower.cooldown <= 0.0
                    && let Some(target) = tower.target
                {
                    let can_fire = if tower.tower_type == TowerType::Sniper {
                        tower.tracking_time >= 2.0
                    } else {
                        true
                    };

                    if can_fire {
                        let enemies_list = self.game_world.resources.enemies_list.clone();
                        if enemies_list.contains(&target)
                            && let (Some(tower_pos), Some(_enemy_pos)) = (
                                self.game_world.get_position(tower_entity).copied(),
                                self.game_world.get_position(target).copied(),
                            )
                        {
                            let engine_entity = spawn_mesh(
                                world,
                                "Sphere",
                                tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                                nalgebra_glm::vec3(0.15, 0.15, 0.15),
                            );

                            let material_name = format!("Projectile_{}", engine_entity.id);
                            material_registry_insert(
                                &mut world.resources.material_registry,
                                material_name.clone(),
                                Material {
                                    base_color: tower.tower_type.color().into(),
                                    emissive_factor: [
                                        tower.tower_type.color().x * 0.5,
                                        tower.tower_type.color().y * 0.5,
                                        tower.tower_type.color().z * 0.5,
                                    ],
                                    ..Default::default()
                                },
                            );
                            if let Some(&index) = world
                                .resources
                                .material_registry
                                .registry
                                .name_to_index
                                .get(&material_name)
                            {
                                world
                                    .resources
                                    .material_registry
                                    .registry
                                    .add_reference(index);
                            }
                            world.set_material_ref(engine_entity, MaterialRef::new(material_name));

                            let projectile_entity = self.game_world.spawn_entities(
                                ENTITY_HANDLE | POSITION | VELOCITY | PROJECTILE,
                                1,
                            )[0];

                            self.game_world
                                .set_entity_handle(projectile_entity, EntityHandle(engine_entity));
                            self.game_world.set_position(
                                projectile_entity,
                                Position(tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0)),
                            );
                            self.game_world.set_velocity(
                                projectile_entity,
                                Velocity(nalgebra_glm::vec3(0.0, 0.0, 0.0)),
                            );
                            self.game_world.set_projectile(
                                projectile_entity,
                                Projectile {
                                    damage: tower.tower_type.damage(),
                                    target,
                                    speed: tower.tower_type.projectile_speed(),
                                    tower_type: tower.tower_type,
                                    start_position: tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                                    arc_height: if tower.tower_type == TowerType::Cannon {
                                        2.0
                                    } else {
                                        0.0
                                    },
                                    flight_progress: 0.0,
                                },
                            );

                            self.game_world
                                .resources
                                .projectiles_list
                                .push(projectile_entity);

                            tower.cooldown = tower.tower_type.fire_rate();
                            tower.fire_animation = 1.0;
                            needs_fire_animation_update = true;
                            animation_scale = 1.0 + tower.fire_animation * 0.2;

                            if tower.tower_type == TowerType::Cannon {
                                self.create_muzzle_flash(
                                    world,
                                    tower_pos.0 + nalgebra_glm::vec3(0.0, 0.8, 0.0),
                                );
                            }
                        }
                    }
                }

                if needs_fire_animation_update
                    && let Some(handle) = self.game_world.get_entity_handle(tower_entity)
                {
                    if let Some(transform) = world.get_local_transform_mut(handle.0) {
                        transform.scale =
                            nalgebra_glm::vec3(0.4 * animation_scale, 0.8, 0.4 * animation_scale);
                        world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                    }

                    if animation_scale > 1.05 {
                        if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                            && let Some(material) = registry_entry_by_name_mut(
                                &mut world.resources.material_registry.registry,
                                &material_ref.name,
                            )
                        {
                            let flash_intensity = (animation_scale - 1.0) * 5.0;
                            material.emissive_factor = [
                                tower.tower_type.color().x * 0.5 + flash_intensity,
                                tower.tower_type.color().y * 0.5 + flash_intensity,
                                tower.tower_type.color().z * 0.5 + flash_intensity,
                            ];
                        }
                    } else if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                        && let Some(material) = registry_entry_by_name_mut(
                            &mut world.resources.material_registry.registry,
                            &material_ref.name,
                        )
                    {
                        material.emissive_factor = [
                            tower.tower_type.color().x * 0.5,
                            tower.tower_type.color().y * 0.5,
                            tower.tower_type.color().z * 0.5,
                        ];
                    }
                }

                self.game_world.set_tower(tower_entity, tower);
            }
        }
    }

    fn projectile_movement_system(&mut self, world: &mut World) {
        let delta_time =
            world.resources.window.timing.delta_time * self.game_world.resources.game_speed;
        let mut projectiles_to_remove = Vec::new();

        for projectile_entity in self.game_world.resources.projectiles_list.clone() {
            if let (Some(projectile), Some(proj_pos)) = (
                self.game_world.get_projectile(projectile_entity).copied(),
                self.game_world.get_position(projectile_entity).copied(),
            ) {
                let enemies_list = self.game_world.resources.enemies_list.clone();
                if !enemies_list.contains(&projectile.target) {
                    projectiles_to_remove.push(projectile_entity);
                    continue;
                }

                if let Some(target_pos) = self.game_world.get_position(projectile.target).copied() {
                    let target_visual_pos =
                        if let Some(enemy) = self.game_world.get_enemy(projectile.target) {
                            target_pos.0 + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0)
                        } else {
                            target_pos.0
                        };

                    let new_pos = if projectile.arc_height > 0.0 {
                        let total_distance =
                            (target_visual_pos - projectile.start_position).magnitude();
                        let mut new_flight_progress = projectile.flight_progress
                            + (projectile.speed * delta_time) / total_distance;
                        new_flight_progress = new_flight_progress.min(1.0);

                        let base_pos = projectile.start_position
                            + (target_visual_pos - projectile.start_position) * new_flight_progress;
                        let arc_offset = 4.0
                            * projectile.arc_height
                            * new_flight_progress
                            * (1.0 - new_flight_progress);
                        base_pos + nalgebra_glm::vec3(0.0, arc_offset, 0.0)
                    } else {
                        let direction = (target_visual_pos - proj_pos.0).normalize();
                        proj_pos.0 + direction * projectile.speed * delta_time
                    };

                    let distance = (target_visual_pos - new_pos).magnitude();
                    let distance_to_target = distance;
                    if distance_to_target < 0.3
                        || (projectile.arc_height > 0.0 && projectile.flight_progress >= 1.0)
                    {
                        if matches!(projectile.tower_type, TowerType::Cannon | TowerType::Sniper) {
                            self.create_explosion_effect(world, target_visual_pos);
                        }

                        if let Some(mut enemy) =
                            self.game_world.get_enemy(projectile.target).copied()
                        {
                            let mut damage_remaining = projectile.damage;

                            if enemy.shield_health > 0.0 {
                                let shield_damage = damage_remaining.min(enemy.shield_health);
                                enemy.shield_health -= shield_damage;
                                damage_remaining -= shield_damage;
                            }

                            if damage_remaining > 0.0 {
                                enemy.health -= damage_remaining;
                            }

                            if matches!(projectile.tower_type, TowerType::Frost) {
                                enemy.slow_duration = 2.0;
                            }

                            if matches!(projectile.tower_type, TowerType::Poison) {
                                enemy.poison_duration = 3.0;
                                enemy.poison_damage = 2.0;
                            }

                            if enemy.health <= 0.0 {
                                self.game_world.resources.money += enemy.value;
                                self.spawn_money_popup(
                                    world,
                                    target_visual_pos,
                                    enemy.value as i32,
                                );
                                self.create_death_effect(world, target_visual_pos);

                                if let Some(idx) = self
                                    .game_world
                                    .resources
                                    .enemies_list
                                    .iter()
                                    .position(|&e| e == projectile.target)
                                {
                                    self.game_world.resources.enemies_list.remove(idx);
                                }
                                self.despawn_entity(world, projectile.target);
                            } else {
                                self.game_world.set_enemy(projectile.target, enemy);
                            }
                        }

                        projectiles_to_remove.push(projectile_entity);
                    } else {
                        if projectile.arc_height > 0.0 {
                            let total_distance =
                                (target_visual_pos - projectile.start_position).magnitude();
                            let mut updated_projectile = projectile;
                            updated_projectile.flight_progress +=
                                (projectile.speed * delta_time) / total_distance;
                            updated_projectile.flight_progress =
                                updated_projectile.flight_progress.min(1.0);
                            self.game_world
                                .set_projectile(projectile_entity, updated_projectile);
                        }

                        self.game_world
                            .set_position(projectile_entity, Position(new_pos));

                        if let Some(handle) = self.game_world.get_entity_handle(projectile_entity)
                            && let Some(transform) = world.get_local_transform_mut(handle.0)
                        {
                            transform.translation = new_pos;
                            world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                        }
                    }
                }
            }
        }

        for entity in projectiles_to_remove {
            if let Some(idx) = self
                .game_world
                .resources
                .projectiles_list
                .iter()
                .position(|&e| e == entity)
            {
                self.game_world.resources.projectiles_list.remove(idx);
            }
            self.despawn_entity(world, entity);
        }
    }
}

impl State for TowerDefenseECS {
    fn title(&self) -> &str {
        "Tower Defense ECS"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = false;
        world.resources.graphics.atmosphere = Atmosphere::Nebula;
        world.resources.graphics.show_grid = false;

        spawn_sun(world);

        let main_camera = world.spawn_entities(
            LOCAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | GLOBAL_TRANSFORM | CAMERA | PAN_ORBIT_CAMERA,
            1,
        )[0];

        let camera_radius = 16.1;
        let camera_pitch = 0.52;
        world.set_local_transform(
            main_camera,
            LocalTransform {
                translation: Vec3::new(0.0, 8.0, 14.0),
                rotation: nalgebra_glm::quat_angle_axis(-0.6, &nalgebra_glm::vec3(1.0, 0.0, 0.0)),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        world.set_local_transform_dirty(main_camera, LocalTransformDirty);
        world.set_global_transform(main_camera, GlobalTransform::default());
        world.set_camera(
            main_camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 45.0_f32.to_radians(),
                    z_near: 0.1,
                    z_far: Some(500.0),
                }),
                smoothing: Some(Smoothing::default()),
            },
        );
        world.set_pan_orbit_camera(
            main_camera,
            PanOrbitCamera {
                focus: Vec3::new(0.0, 0.0, 0.0),
                target_focus: Vec3::new(0.0, 0.0, 0.0),
                radius: camera_radius,
                target_radius: camera_radius,
                pitch: camera_pitch,
                target_pitch: camera_pitch,
                yaw: 0.0,
                target_yaw: 0.0,
                ..Default::default()
            },
        );
        world.resources.active_camera = Some(main_camera);

        self.game_world.resources.camera_entity = main_camera;

        self.game_world.resources.money = 200;
        self.game_world.resources.lives = 20;
        self.game_world.resources.wave = 0;
        self.game_world.resources.game_state = GameState::WaitingForWave;
        self.game_world.resources.selected_tower_type = TowerType::Basic;
        self.game_world.resources.spawn_timer = 0.0;
        self.game_world.resources.wave_delay = 3.0;
        self.game_world.resources.wave_announce_timer = 0.0;
        self.game_world.resources.game_speed = 1.0;
        self.game_world.resources.current_hp = 20;
        self.game_world.resources.max_hp = 20;
        self.game_world.resources.auto_start_waves = false;

        self.initialize_grid();
        self.create_path(world);
        self.spawn_grid_tiles(world);

        let mut ui_handles = UiHandles {
            money_text: Some(spawn_3d_text_with_properties(
                world,
                &format!("Money: ${}", self.game_world.resources.money),
                nalgebra_glm::vec3(-10.0, 2.0, -8.0),
                TextProperties {
                    font_size: 80.0,
                    color: nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0),
                    alignment: TextAlignment::Left,
                    outline_width: 0.1,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            lives_text: Some(spawn_3d_text_with_properties(
                world,
                &format!("Lives: {}", self.game_world.resources.lives),
                nalgebra_glm::vec3(-1.0, 2.0, -8.0),
                TextProperties {
                    font_size: 60.0,
                    color: nalgebra_glm::vec4(1.0, 0.2, 0.2, 1.0),
                    alignment: TextAlignment::Center,
                    outline_width: 0.1,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            hp_text: Some(spawn_3d_text_with_properties(
                world,
                &format!(
                    "HP: {}/{}",
                    self.game_world.resources.current_hp, self.game_world.resources.max_hp
                ),
                nalgebra_glm::vec3(3.0, 1.0, -8.0),
                TextProperties {
                    font_size: 50.0,
                    color: nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0),
                    alignment: TextAlignment::Left,
                    outline_width: 0.08,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            wave_text: Some(spawn_3d_text_with_properties(
                world,
                &format!("Wave: {}", self.game_world.resources.wave),
                nalgebra_glm::vec3(10.0, 2.0, -8.0),
                TextProperties {
                    font_size: 80.0,
                    color: nalgebra_glm::vec4(0.2, 0.8, 1.0, 1.0),
                    alignment: TextAlignment::Right,
                    outline_width: 0.1,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            status_text: Some(spawn_3d_text_with_properties(
                world,
                "",
                nalgebra_glm::vec3(0.0, 4.0, 0.0),
                TextProperties {
                    font_size: 160.0,
                    color: nalgebra_glm::vec4(1.0, 1.0, 1.0, 1.0),
                    alignment: TextAlignment::Center,
                    outline_width: 0.12,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            wave_announce_text: Some(spawn_3d_text_with_properties(
                world,
                "",
                nalgebra_glm::vec3(0.0, 3.0, 0.0),
                TextProperties {
                    font_size: 120.0,
                    color: nalgebra_glm::vec4(1.0, 0.8, 0.0, 1.0),
                    alignment: TextAlignment::Center,
                    outline_width: 0.15,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    smoothing: 0.2,
                    ..Default::default()
                },
            )),
            lives_bar: None,
            lives_bar_bg: None,
            tower_select_texts: Vec::new(),
        };

        for (index, tower_type) in TowerType::all().iter().enumerate() {
            let is_selected = *tower_type == self.game_world.resources.selected_tower_type;
            let base_color = tower_type.color();
            let text_color = if is_selected {
                nalgebra_glm::vec4(1.0, 1.0, 0.0, 1.0)
            } else {
                nalgebra_glm::vec4(base_color.x, base_color.y, base_color.z, 0.7)
            };

            let text = format!(
                "[{}] {} - ${}",
                index + 1,
                tower_type.name(),
                tower_type.cost()
            );

            let entity = spawn_hud_text_with_properties(
                world,
                &text,
                HudAnchor::TopLeft,
                nalgebra_glm::vec2(15.0, 15.0 + (index as f32 * 28.0)),
                TextProperties {
                    font_size: 22.0,
                    color: text_color,
                    alignment: TextAlignment::Left,
                    outline_width: 0.02,
                    outline_color: nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                },
            );
            ui_handles.tower_select_texts.push(entity);
        }

        if let Some(status_text) = ui_handles.status_text
            && let Some(visibility) = world.get_visibility_mut(status_text)
        {
            visibility.visible = false;
        }

        if let Some(wave_announce_text) = ui_handles.wave_announce_text
            && let Some(visibility) = world.get_visibility_mut(wave_announce_text)
        {
            visibility.visible = false;
        }

        ui_handles.lives_bar_bg = Some(spawn_mesh(
            world,
            "Cube",
            nalgebra_glm::vec3(-1.0, 1.0, -8.0),
            nalgebra_glm::vec3(6.0, 0.3, 0.1),
        ));

        if let Some(bg_entity) = ui_handles.lives_bar_bg {
            let material_name = format!("LivesBarBg_{}", bg_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.2, 0.2, 0.2, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(bg_entity, MaterialRef::new(material_name));
        }

        let total_hp = (self.game_world.resources.lives - 1) * self.game_world.resources.max_hp
            + self.game_world.resources.current_hp;
        let max_total_hp = self.game_world.resources.lives * self.game_world.resources.max_hp;
        let health_percentage = total_hp as f32 / max_total_hp as f32;
        let bar_width = 5.8 * health_percentage;

        ui_handles.lives_bar = Some(spawn_mesh(
            world,
            "Cube",
            nalgebra_glm::vec3(-3.9 + bar_width / 2.0, 1.0, -7.9),
            nalgebra_glm::vec3(bar_width, 0.25, 0.1),
        ));

        if let Some(bar_entity) = ui_handles.lives_bar {
            let material_name = format!("LivesBar_{}", bar_entity.id);
            material_registry_insert(
                &mut world.resources.material_registry,
                material_name.clone(),
                Material {
                    base_color: [0.0, 1.0, 0.0, 1.0],
                    ..Default::default()
                },
            );
            if let Some(&index) = world
                .resources
                .material_registry
                .registry
                .name_to_index
                .get(&material_name)
            {
                world
                    .resources
                    .material_registry
                    .registry
                    .add_reference(index);
            }
            world.set_material_ref(bar_entity, MaterialRef::new(material_name));
        }

        self.game_world.resources.ui_handles = ui_handles;
    }

    fn run_systems(&mut self, world: &mut World) {
        escape_key_exit_system(world);
        pan_orbit_camera_system(world);

        self.input_system(world);
        self.tile_hover_system(world);
        self.placement_preview_system(world);
        self.range_indicator_system(world);

        self.wave_spawning_system(world);
        self.enemy_movement_system(world);
        self.tower_targeting_system(world);
        self.tower_shooting_system(world);
        self.projectile_movement_system(world);

        let delta_time =
            world.resources.window.timing.delta_time * self.game_world.resources.game_speed;
        self.update_visual_effects(world, delta_time);
        self.update_money_popups(world, world.resources.window.timing.delta_time);
        self.ui_update_system(world);

        update_particle_emitters(world, world.resources.window.timing.delta_time);
        nightshade::ecs::text::systems::sync_text_meshes_system(world);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let particle_pass = passes::ParticlePass::new(device, wgpu::TextureFormat::Rgba16Float);
        graph
            .pass(Box::new(particle_pass))
            .slot("color", resources.scene_color)
            .slot("depth", resources.depth);

        let (width, height) = (1920, 1080);
        let bloom_width = width / 2;
        let bloom_height = height / 2;

        let bloom_texture = graph
            .add_color_texture("bloom")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(bloom_width, bloom_height)
            .clear_color(wgpu::Color::BLACK)
            .transient();

        let bloom_pass = passes::BloomPass::new(device, width, height);
        graph
            .pass(Box::new(bloom_pass))
            .read("hdr", resources.scene_color)
            .write("bloom", bloom_texture);

        let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.005);
        graph
            .pass(Box::new(postprocess_pass))
            .read("hdr", resources.scene_color)
            .read("bloom", bloom_texture)
            .write("output", resources.swapchain);
    }

    fn on_keyboard_input(&mut self, world: &mut World, key: KeyCode, state: KeyState) {
        if state != KeyState::Pressed {
            return;
        }

        if key == KeyCode::KeyC || key == KeyCode::Home {
            let camera = self.game_world.resources.camera_entity;
            if let Some(pan_orbit) = world.get_pan_orbit_camera_mut(camera) {
                pan_orbit.target_focus = Vec3::new(0.0, 0.0, 0.0);
                pan_orbit.target_radius = 16.1;
                pan_orbit.target_pitch = 0.52;
                pan_orbit.target_yaw = 0.0;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EntityHandle(pub Entity);

#[derive(Debug, Clone, Copy, Default)]
pub struct Position(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Tower {
    pub tower_type: TowerType,
    pub cooldown: f32,
    pub target: Option<freecs::Entity>,
    pub fire_animation: f32,
    pub tracking_time: f32,
    pub target_line: Option<Entity>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GridCell {
    pub x: i32,
    pub z: i32,
    pub occupied: bool,
    pub is_path: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RangeIndicator {
    pub tower_entity: freecs::Entity,
    pub line_entities: Vec<Entity>,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MoneyPopup {
    pub text_entity: Entity,
    pub lifetime: f32,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub struct Enemy {
    pub health: f32,
    pub max_health: f32,
    pub shield_health: f32,
    pub max_shield: f32,
    pub speed: f32,
    pub path_index: usize,
    pub path_progress: f32,
    pub value: u32,
    pub enemy_type: EnemyType,
    pub slow_duration: f32,
    pub poison_duration: f32,
    pub poison_damage: f32,
    pub is_flying: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Projectile {
    pub damage: f32,
    pub target: freecs::Entity,
    pub speed: f32,
    pub tower_type: TowerType,
    pub start_position: Vec3,
    pub arc_height: f32,
    pub flight_progress: f32,
}

#[derive(Debug, Clone, Default)]
pub struct HealthBar {
    pub enemy_entity: freecs::Entity,
    pub bar_entity: Entity,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EffectType {
    #[default]
    Explosion,
    PoisonBubble,
    DeathParticle,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VisualEffect {
    pub effect_type: EffectType,
    pub lifetime: f32,
    pub age: f32,
}
