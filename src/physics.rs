use crate::{render::TilemapState, PlayerState};

#[derive(Default, Clone)]
pub struct Collider {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone)]
pub struct PhysicsState {
    pub dt: f32,
    pub dt_timer: std::time::Instant,
    pub colliders: Vec<Collider>,
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            dt: 0.0,
            dt_timer: std::time::Instant::now(),
            colliders: vec![],
        }
    }
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

pub fn player_physics(state: &mut PhysicsState, player: &mut PlayerState) {
    let now = std::time::Instant::now();
    let dt = (now.duration_since(state.dt_timer).as_secs_f64()) as f32;
    state.dt_timer = std::time::Instant::now();

    let mut obj = player;

    let max_speed: f32 = 150.0;
    let fri: f32 = 500.0;
    let min_fri: f32 = 10.0;
    let accel: f32 = 500.0 + fri;
    let gravity: f32 = 800.0;
    let coyote_time: f32 = 0.1;
    let jump_buffer_time: f32 = 0.1;

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

    if obj.wants_to_jump{
        obj.jump_buffer_counter = jump_buffer_time;
    } else {
        obj.jump_buffer_counter -= dt;
    }

    let jump_force = 350.0;
    if obj.jump_buffer_counter > 0.0 && obj.coyote_time_counter > 0.0 {
        obj.velocity.y = -jump_force;
        obj.wants_to_jump = false;
        obj.coyote_time_counter = 0.0;
        obj.jump_buffer_counter = 0.0;
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
    let ny = obj.y + obj.velocity.y * dt; // new y

    let ox = nx + obj.hitbox.x as f32; // for colliding purposes
    let oy = ny + obj.hitbox.y as f32;
    let ow = obj.hitbox.w;
    let oh = obj.hitbox.h;

    obj.is_grounded = false;
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
        }
        if (obj.x as i32 + ow as i32) > (col.x as i32)
            && (col.x as i32 + col.w as i32) > (obj.x as i32)
            && (oy as i32) < (col.y as i32 + col.h as i32)
            && (oy as i32 + oh as i32) > (col.y as i32)
        {
            is_colliding_y = true;
            obj.velocity.y = 0.0;
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
}
