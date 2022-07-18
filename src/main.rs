extern crate sdl2;

use std::collections::HashMap;

use glam::Vec2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

mod render;
use crate::render::*;
mod physics;
use crate::physics::*;

struct InputState {
    event_pump: EventPump,
    should_quit: bool,
    key_pressed_state: HashMap<Keycode, bool>,
    key_released_state: HashMap<Keycode, bool>,
    key_state: HashMap<Keycode, bool>,
}

#[derive(Clone)]
pub struct PlayerState {
    _player_sprite_path: &'static str,
    _anim_x: u32,
    _anim_y: u32,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    hitbox: Collider,
    wants_dir: f32,
    added_velocity: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    wants_to_jump: bool,
    is_grounded: bool,
}

impl PlayerState {
    fn new(x: f32, y: f32) -> Self {
        Self {
            _player_sprite_path: "res/player.png",
            width: 8,
            height: 16,
            _anim_x: 0,
            _anim_y: 0,
            x:x,
            y:y,
            hitbox: Collider {
                x: 0,
                y: 0,
                w: 8,
                h: 16,
            },
            wants_dir: 0.0,
            added_velocity: Vec2::ZERO,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            wants_to_jump: false,
            is_grounded: false,
        }
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Project tomato", 1280, 720)
        .position_centered()
        //.fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas
        .set_logical_size(320, 180)
        .map_err(|e| e.to_string())?; // set later for resolution (320, 180) for 16:9 and (320, 240) for 4:3 etc
    let event_pump = sdl_context.event_pump()?;

    let mut rendering_state = RenderingState::new(canvas);
    let mut input_state = InputState {
        event_pump,
        should_quit: false,
        key_pressed_state: HashMap::new(),
        key_released_state: HashMap::new(),
        key_state: HashMap::new(),
    };

    let pool = threadpool::ThreadPool::new(num_cpus::get());
    let (send_player_physics, recv_player_physics) = std::sync::mpsc::channel();

    let mut loader = tiled::Loader::new();
    let mut start_map = loader.load_tmx_map("res/startmenu.tmx").unwrap();
    load_tilemap_to_textures(&mut rendering_state, &mut start_map);

    let (x,y) =  get_player_spawn_position(&start_map);
    let mut player_state = PlayerState::new(x, y);
    let mut physics_state = PhysicsState::default();
    load_tilemap_to_physics(&mut physics_state, &start_map);

    loop {
        let frame_timer = std::time::Instant::now();
        let render_player_state = player_state.clone();
        let render_physics_state = physics_state.clone();

        input(&mut input_state);
        if input_state.should_quit {
            break;
        };

        move_player(&mut player_state, &input_state);

        let sender = send_player_physics.clone();
        pool.execute(move || {
            player_physics(&mut physics_state, &mut player_state);
            sender.send((physics_state, player_state)).unwrap();
        });

        render(
            &mut rendering_state,
            &mut start_map,
            &render_player_state,
            &render_physics_state,
        );

        pool.join();
        (physics_state, player_state) = recv_player_physics.recv().unwrap();

        let _frame_end_time = frame_timer.elapsed();
        //println!("{}", 1.0/_frame_end_time.as_secs_f64());
        std::thread::sleep(std::time::Duration::from_millis(16)); // ONLY FOR DEV PURPOSES
    }

    Ok(())
}

fn get_player_spawn_position(tile: &TilemapState) -> (f32, f32){
    for layer in tile.layers(){
        if layer.name == "PlayerSpawners"{
            match layer.layer_type(){
                tiled::LayerType::TileLayer(_) => {},
                tiled::LayerType::ObjectLayer(obj) => {
                    for o in obj.objects(){
                        if o.name == "PlayerSpawn"{
                            return (o.x, o.y - 16.0); // obj origin is bottom left in tiled whereas top left in sdl
                        }
                    }
                },
                tiled::LayerType::ImageLayer(_) => {},
                tiled::LayerType::GroupLayer(_) => {},
            }
        }
    }
    panic!("PLAYER SPAWN NOT FOUND");
}

fn move_player(player: &mut PlayerState, input: &InputState) {
    let mut wanna_move = false;
    if get_key(sdl2::keyboard::Keycode::Left, input) {
        player.wants_dir = -1.0;
        wanna_move = true;
    }
    if get_key(sdl2::keyboard::Keycode::Right, input) {
        player.wants_dir = 1.0;
        wanna_move = true;
    }
    if get_key_pressed(sdl2::keyboard::Keycode::Z, input) {
        player.wants_to_jump = true;
    } else {
        player.wants_to_jump = false;
    }
    if !wanna_move {
        player.wants_dir = 0.0;
    }
}

fn input(state: &mut InputState) {
    let event_pump = &mut state.event_pump;
    state.key_pressed_state.clear();
    state.key_released_state.clear();
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                state.should_quit = true;
                return;
            }
            Event::KeyDown { keycode: key, .. } => {
                state.key_pressed_state.insert(key.unwrap(), true);
                state.key_state.insert(key.unwrap(), true);
            }
            Event::KeyUp { keycode: key, .. } => {
                state.key_released_state.insert(key.unwrap(), false);
                state.key_state.insert(key.unwrap(), false);
            }
            _ => {}
        }
    }
    if get_key(Keycode::Escape, state) {
        state.should_quit = true;
    }
}

fn get_key(key: sdl2::keyboard::Keycode, input: &InputState) -> bool {
    if let Some(s) = input.key_state.get(&key) {
        return *s;
    } else {
        return false;
    };
}

fn get_key_pressed(key: sdl2::keyboard::Keycode, input: &InputState) -> bool {
    if let Some(s) = input.key_pressed_state.get(&key) {
        return *s;
    } else {
        return false;
    };
}
