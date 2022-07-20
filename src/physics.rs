use tiled::{Properties, PropertyValue};

use crate::{render::TilemapState, PlayerState};

#[derive(Default, Clone)]
pub struct Collider {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone)]
pub enum Interactions {
    /// Map path and spawn number
    ChangeMap(String, u32),
}

#[derive(Clone)]
pub struct Interactable {
    pub collider: Collider,
    pub interaction: Interactions,
    pub is_in_collider: bool,
}

#[derive(Clone)]
pub struct PhysicsState {
    pub dt: f32,
    pub dt_timer: std::time::Instant,
    pub colliders: Vec<Collider>,
    pub interactables: Vec<Interactable>,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            dt: 0.0,
            dt_timer: std::time::Instant::now(),
            colliders: vec![],
            interactables: vec![],
        }
    }
}

pub fn load_tilemap_to_interactables(state: &mut PhysicsState, tile_state: &TilemapState) {
    let mut interactables: Vec<Interactable> = vec![];
    for layer in tile_state.layers() {
        if layer.name == "Interactables" {
            match layer.layer_type() {
                tiled::LayerType::ObjectLayer(obj_layer) => {
                    for obj in obj_layer.objects() {
                        match obj.shape {
                            tiled::ObjectShape::Rect { width, height } => {
                                let x = obj.x as i32;
                                let y = obj.y as i32;
                                let w = width as u32;
                                let h = height as u32;
                                let col = Collider { x, y, w, h };

                                if let Some(map_change) = obj.properties.get("map change") {
                                    let map_path =
                                        if let PropertyValue::StringValue(path) = map_change {
                                            path
                                        } else {
                                            panic!()
                                        };
                                    let spawn_place = if let PropertyValue::IntValue(spawn_place) =
                                        obj.properties.get("spawn place").unwrap()
                                    {
                                        spawn_place
                                    } else {
                                        panic!()
                                    };

                                    let interaction = Interactions::ChangeMap(
                                        map_path.clone(),
                                        *spawn_place as u32,
                                    );
                                    interactables.push(Interactable {
                                        collider: col,
                                        interaction: interaction,
                                        is_in_collider: false,
                                    })
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    state.interactables.append(&mut interactables);
}

pub fn load_tilemap_to_physics(state: &mut PhysicsState, tile_state: &TilemapState) {
    let mut colliders: Vec<Collider> = vec![];
    for layer in tile_state.layers() {
        if layer.name == "Colliders" {
            match layer.layer_type() {
                tiled::LayerType::ObjectLayer(obj_layer) => {
                    for obj in obj_layer.objects() {
                        match obj.shape {
                            tiled::ObjectShape::Rect { width, height } => {
                                let x = obj.x as i32;
                                let y = obj.y as i32;
                                let w = width as u32;
                                let h = height as u32;
                                let col = Collider { x, y, w, h };
                                colliders.push(col);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    state.colliders.append(&mut colliders);
}

pub fn player_collision_interactables(physics: &mut PhysicsState, player: &mut PlayerState) {
    player.can_interact = false;
    for interactable in physics.interactables.iter_mut() {
        let col = &interactable.collider;
        if is_colliding(
            player.x as i32,
            player.y as i32,
            player.width as i32,
            player.height as i32,
            col.x,
            col.y,
            col.w as i32,
            col.h as i32,
        ) {
            interactable.is_in_collider = true;
            player.can_interact = true;
        } else {
            interactable.is_in_collider = false;
        }
    }
}

fn is_colliding(x1: i32, y1: i32, w1: i32, h1: i32, x2: i32, y2: i32, w2: i32, h2: i32) -> bool {
    return x1 < x2 + w2 && x1 + w1 > x2 && y1 + h1 > y2 && y1 < y2 + h2;
}

pub fn player_physics(state: &mut PhysicsState, player: &mut PlayerState) {
    let now = std::time::Instant::now();
    let dt = (now.duration_since(state.dt_timer).as_secs_f64()) as f32;
    state.dt_timer = std::time::Instant::now();

    let mut obj = player;

    let max_speed: f32 = 150.0;
    let sliding_speed: f32 = 100.0;
    let fri: f32 = 500.0;
    let min_fri: f32 = 10.0;
    let accel: f32 = 500.0 + fri;
    let gravity: f32 = 800.0;
    let coyote_time: f32 = 0.1;
    let jump_buffer_time: f32 = 0.1;
    let wall_jump_force = 300.0;
    let jump_force = 350.0;

    // MOVE LEFT-RIGHT = ADDED_VELOCITY
    // JUMP AND GRAVITY = VELOCITY

    obj.acceleration.x = obj.wants_dir * accel;
    obj.added_velocity.x += obj.acceleration.x * dt;
    obj.added_velocity.x = obj.added_velocity.x.clamp(-max_speed, max_speed);

    obj.velocity.y += gravity * dt;

    if obj.is_grounded {
        obj.coyote_time_counter = coyote_time;
    } else {
        obj.coyote_time_counter -= dt;
    }

    if obj.wants_to_jump {
        obj.jump_buffer_counter = jump_buffer_time;
    } else {
        obj.jump_buffer_counter -= dt;
    }

    //jumping
    if obj.jump_buffer_counter > 0.0 && obj.coyote_time_counter > 0.0 {
        obj.velocity.y = -jump_force;
        obj.wants_to_jump = false;
        obj.coyote_time_counter = 0.0;
        obj.jump_buffer_counter = 0.0;
    }
    // wall jumping
    if obj.jump_buffer_counter > 0.0 && obj.is_sliding {
        obj.added_velocity.x = -obj.wants_dir * wall_jump_force;
        obj.velocity.y = -jump_force;
        obj.wants_to_jump = false;
    }

    if obj.is_sliding {
        if obj.velocity.y > sliding_speed {
            obj.velocity.y = sliding_speed;
        }
    }

    // friction
    if obj.added_velocity.x > 0.0 {
        obj.added_velocity.x -= fri * dt;
    }
    if obj.added_velocity.x < 0.0 {
        obj.added_velocity.x += fri * dt;
    }
    if obj.added_velocity.x.abs() < min_fri && obj.wants_dir == 0.0 {
        obj.added_velocity.x = 0.0;
    }

    let nx = obj.x + obj.added_velocity.x * dt; // new x
    let ny = obj.y + (obj.added_velocity.y + obj.velocity.y) * dt; // new y

    let ox = nx + obj.hitbox.x as f32; // for colliding purposes
    let oy = ny + obj.hitbox.y as f32;
    let ow = obj.hitbox.w;
    let oh = obj.hitbox.h;

    obj.is_grounded = false;
    obj.is_sliding = false;
    obj.is_falling = true;
    let mut is_colliding_x = false;
    let mut is_colliding_y = false;

    for col in state.colliders.iter() {
        if (ox as i32 + ow as i32) > (col.x as i32)
            && (col.x as i32 + col.w as i32) > (ox as i32)
            && (obj.y as i32) < (col.y as i32 + col.h as i32)
            && (obj.y as i32 + oh as i32) > (col.y as i32)
        {
            is_colliding_x = true;
            obj.velocity.x = 0.0;
            if !obj.is_grounded {
                obj.is_sliding = true;
            }
        }
        if (obj.x as i32 + ow as i32) > (col.x as i32)
            && (col.x as i32 + col.w as i32) > (obj.x as i32)
            && (oy as i32) < (col.y as i32 + col.h as i32)
            && (oy as i32 + oh as i32) > (col.y as i32)
        {
            is_colliding_y = true;
            obj.velocity.y = 0.0;
            obj.is_falling = false;
            if oy as i32 >= obj.y as i32 {
                obj.is_grounded = true;
            }
        }
    }

    if !is_colliding_x {
        obj.x = nx;
    }
    if !is_colliding_y {
        obj.y = ny;
    }
    player_collision_interactables(state, obj);
}
