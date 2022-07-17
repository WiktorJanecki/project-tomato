extern crate sdl2;

use std::collections::HashMap;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod render;
use crate::render::*;
mod physics;
use crate::physics::*;

struct InputState{
    event_pump: EventPump, 
    should_quit: bool,
    key_pressed_state: HashMap<Keycode, bool>,
    key_released_state: HashMap<Keycode, bool>,
    key_state: HashMap<Keycode, bool>,
}

#[derive(Clone)]
pub struct PlayerState{
    player_sprite_path: &'static str,
    width: u32,
    height: u32,
    anim_x: u32,
    anim_y: u32,
    x: f32,
    y: f32,
    friction: f32,
    hitbox: Collider,
    wants_dir: f32,
    added_velocity: f32,
    acceleration: f32,
    acceleration_y: f32,
    velocity_y: f32,
    wants_to_jump: bool,
    is_grounded:bool,
}


pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Project tomato",1280,720)
        .position_centered()
        //.fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(320, 180).map_err(|e| e.to_string())?; // set later for resolution (320, 180) for 16:9 and (320, 240) for 4:3 etc
    let event_pump = sdl_context.event_pump()?;

    let mut rendering_state = RenderingState::new(canvas);
    let mut input_state = InputState{event_pump, should_quit:false,key_pressed_state:HashMap::new(), key_released_state:HashMap::new(), key_state:HashMap::new()};

    let _pool = threadpool::ThreadPool::new(num_cpus::get());

    let mut loader = tiled::Loader::new();
    let mut start_map = loader.load_tmx_map("res/startmenu.tmx").unwrap();
    load_tilemap_to_textures(&mut rendering_state, &mut start_map);

    let mut player_state = PlayerState{acceleration_y: 0.0,is_grounded: false,wants_to_jump: false,velocity_y: 0.0,player_sprite_path: "player.png", width:8, height:16, anim_x:0,anim_y:0,wants_dir:0.0, added_velocity:0.0, x: 30.0, y: 100.0, acceleration:0.0, friction: 1.0, hitbox: Collider { x: 0, y: 0, w: 8, h: 16 }};
    let mut physics_state = PhysicsState::default();
    load_tilemap_to_physics(&mut physics_state, &start_map);

    loop{
        let frame_timer = std::time::Instant::now();
        let render_player_state = player_state.clone();
        let render_physics_state = physics_state.clone();

        input(&mut input_state);
        move_player(&mut player_state, &input_state);
        if input_state.should_quit { break };
        let handle = std::thread::spawn(move||{
            physics(&mut physics_state,&mut player_state);
            return (physics_state, player_state);
        });
        render(&mut rendering_state, &mut start_map, &render_player_state, &render_physics_state);
        let (x, y) = handle.join().unwrap();
        physics_state = x;
        player_state = y;

        let frame_end_time = frame_timer.elapsed();
        std::thread::sleep(std::time::Duration::from_millis(16));
        //println!("{}", 1.0/frame_end_time.as_secs_f64());
    }

    Ok(())
}

fn load_tilemap_to_physics(state: &mut PhysicsState, tile_state: &TilemapState){
    let mut colliders: Vec<Collider> = vec![];
    for layer in tile_state.layers(){
        match layer.layer_type(){
            tiled::LayerType::ObjectLayer(obj_layer) => {
                for obj in obj_layer.objects(){
                    match obj.shape{
                        tiled::ObjectShape::Rect { width, height } => {
                            let x = obj.x as i32;
                            let y = obj.y as i32;
                            let w = width as u32;
                            let h = height as u32;
                            let col = Collider{x,y,w,h};
                            colliders.push(col);
                        },
                        _=>{},
                    }
                }
            },
            _ => {}
        }
    }
    state.colliders.append(&mut colliders);
}

fn physics(state: &mut PhysicsState, player: &mut PlayerState){
    let now = std::time::Instant::now();
    let dt = (now.duration_since(state.dt_timer).as_secs_f64()) as f32;
    state.dt_timer = std::time::Instant::now();

    let mut obj = player;

    // TODO: FALLING FRICTION coyote time and jump buffering
    
    const max_speed: f32 = 150.0;
    const fri: f32 = 500.0;
    const min_fri: f32 = 10.0;
    const accel: f32 = 500.0 + fri;
    obj.acceleration = obj.wants_dir * accel; 
    obj.added_velocity += obj.acceleration * dt;
    if obj.added_velocity > max_speed {obj.added_velocity = max_speed}
    if obj.added_velocity < -max_speed {obj.added_velocity = -max_speed}
    
    const gravity: f32 = 800.0;
    obj.velocity_y += gravity * dt;

    let mut is_colliding_x = false;
    let mut is_colliding_y = false;

    let jump_force = 350.0;
    if obj.wants_to_jump && obj.is_grounded {
        obj.velocity_y = -jump_force;
        obj.wants_to_jump = false;
    }

    // friction
    if obj.added_velocity > 0.0{
        obj.added_velocity -= fri * dt;
    }
    if obj.added_velocity < 0.0{
        obj.added_velocity += fri*dt;
    }
    if obj.added_velocity.abs() < min_fri && obj.wants_dir == 0.0{
        obj.added_velocity = 0.0;
    }

    let nx = obj.x + obj.added_velocity * dt;
    let ny = obj.y + obj.velocity_y*dt;
    let ox = nx + obj.hitbox.x as f32;
    let oy = ny + obj.hitbox.y as f32;
    let ow = obj.hitbox.w;
    let oh = obj.hitbox.h;

    obj.is_grounded = false;

    for col in state.colliders.iter(){
        if  (ox as i32 + ow as i32) > (col.x as i32) && (col.x as i32 + col.w as i32) > (ox as i32)
         && (obj.y as i32)  < (col.y as i32 + col.h as i32) && (obj.y as i32 + oh as i32) > (col.y as i32){
                is_colliding_x = true;
            }
        if  (obj.x as i32 + ow as i32) > (col.x as i32) && (col.x as i32 + col.w as i32) > (obj.x as i32)
         && (oy as i32)  < (col.y as i32 + col.h as i32) && (oy as i32 + oh as i32) > (col.y as i32){
                is_colliding_y = true;
                if oy as i32 >= obj.y as i32{
                    obj.is_grounded = true;
                    obj.velocity_y = 0.0;
                }
            }
    }


    if !is_colliding_x{obj.x = nx;}
    if !is_colliding_y{obj.y = ny;}

}
fn move_player(player: &mut PlayerState, input: &InputState){
    let mut wanna_move = false;
    if get_key(sdl2::keyboard::Keycode::Left, input){
        player.wants_dir = -1.0;
        wanna_move = true;
    }
    if get_key(sdl2::keyboard::Keycode::Right, input){
        player.wants_dir = 1.0;
        wanna_move = true;
    }
    if get_key_pressed(sdl2::keyboard::Keycode::Up, input){
        player.wants_to_jump = true;    
    } 
    else{
        player.wants_to_jump = false;
    }
    if !wanna_move{
        player.wants_dir = 0.0;
    }
}

fn input(state: &mut InputState){
    let event_pump = &mut state.event_pump;
    state.key_pressed_state.clear();
    state.key_released_state.clear();
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {state.should_quit = true; return;},
            Event::KeyDown {
                keycode: key,
                ..
            } =>{
                state.key_pressed_state.insert(key.unwrap(), true);
                state.key_state.insert(key.unwrap(), true);
            },
            Event::KeyUp { keycode: key, .. } =>{
                state.key_released_state.insert(key.unwrap(), false);
                state.key_state.insert(key.unwrap(), false);
            }
            _ => {}
        }
    }
}

fn get_key(key: sdl2::keyboard::Keycode, input: &InputState) -> bool{
    if let Some(s) = input.key_state.get(&key) { return *s } else{ return false};
}

fn get_key_pressed(key: sdl2::keyboard::Keycode, input: &InputState) -> bool{
    if let Some(s) = input.key_pressed_state.get(&key) { return *s } else{ return false};
}
