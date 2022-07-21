extern crate sdl2;

use r_i18n::I18n;
use r_i18n::I18nConfig;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use std::collections::HashMap;

mod render;
use crate::render::*;
mod physics;
use crate::physics::*;
mod enemy;
use crate::enemy::*;
mod player;
use crate::player::*;

struct InputState {
    event_pump: EventPump,
    should_quit: bool,
    key_pressed_state: HashMap<Keycode, bool>,
    key_released_state: HashMap<Keycode, bool>,
    key_state: HashMap<Keycode, bool>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Project tomato", 1280, 720)
        .position_centered()
        .fullscreen_desktop()
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
    let mut player_state = PlayerState::new(0.0, 0.0);
    let mut physics_state = PhysicsState::default();
    let mut enemies_state = EnemiesState::new();
    let mut animation_state = AnimationState::new();

    let i18n_config: I18nConfig = I18nConfig {
        locales: &["en", "pl"],
        directory: "res/translations/",
    };
    let mut lang: I18n = I18n::configure(&i18n_config);
    lang.set_current_lang("en");

    let mut loader = tiled::Loader::new();
    let mut start_map = switch_map(
        &mut loader,
        "res/startmenu.tmx",
        0,
        &lang,
        &mut rendering_state,
        &mut player_state,
        &mut enemies_state,
        &mut physics_state,
    );

    // -------------------- GAME LOOP -------------------- //

    loop {
        let frame_timer = std::time::Instant::now();
        let render_player_state = player_state.clone();
        let render_physics_state = physics_state.clone();
        let render_enemies_state = enemies_state.clone();

        input(&mut input_state);
        if input_state.should_quit {
            break;
        };

        move_player(&mut player_state, &input_state);
        animate(&mut animation_state, &mut player_state, &mut enemies_state);
        count_dt(&mut physics_state);
        player_physics(&physics_state, &mut player_state);
        enemies_physics(&physics_state, &mut enemies_state);
        player_collision_interactables(&mut physics_state, &mut player_state);

        render(
            &mut rendering_state,
            &mut lang,
            &mut start_map,
            &render_player_state,
            &render_enemies_state,
            &render_physics_state,
        );

        let interaction_result = player_interact(
            &mut loader,
            &lang,
            &mut rendering_state,
            &mut player_state,
            &mut enemies_state,
            &mut physics_state,
        );
        match interaction_result {
            InteractionResult::Nothing => {}
            InteractionResult::ChangeMap(new_map) => start_map = new_map,
        }

        let _frame_end_time = frame_timer.elapsed();
        //println!("{}", 1.0/_frame_end_time.as_secs_f64());
        std::thread::sleep(std::time::Duration::from_millis(16)); // ONLY FOR DEV PURPOSES
    }

    Ok(())
}

fn switch_map(
    loader: &mut tiled::Loader,
    path: &str,
    spawn_number: u32,
    lang: &I18n,
    render: &mut RenderingState,
    player: &mut PlayerState,
    enemies: &mut EnemiesState,
    physics: &mut PhysicsState,
) -> tiled::Map {
    let map = loader.load_tmx_map(path).unwrap();

    render.text_hints.clear();
    physics.colliders.clear();
    physics.interactables.clear();

    load_tilemap_to_textures(render, &map);
    load_tilemap_to_text_hints(render, &map, &lang);
    load_tilemap_to_physics(physics, &map);
    load_tilemap_to_interactables(physics, &map);
    load_tilemap_to_enemies(enemies, &map, render);
    load_player_spawn(player, &map, spawn_number);
    return map;
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
    player.wants_to_interact = get_key(sdl2::keyboard::Keycode::Up, input);
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
