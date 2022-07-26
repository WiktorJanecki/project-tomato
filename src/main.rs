extern crate sdl2;

use fontdue::layout::Layout;
use fontdue::layout::LayoutSettings;
use fontdue::layout::TextStyle;
use json::JsonValue;
use r_i18n::I18n;
use r_i18n::I18nConfig;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
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

pub struct DialogState {
    color: Color,
    font: usize,
    layout: Layout<Color>,
    current_char: usize,
    text: String,
    texts: Vec<String>,
    finished: bool,
    show: bool,
    dialogues: JsonValue,
}

impl DialogState {
    pub fn new() -> Self {
        Self {
            color: Color::GREEN,
            font: 0,
            current_char: 0,
            layout: Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown),
            text: "".to_owned(),
            texts: vec![], //vec!["asdf".to_owned(), "dfasdgasdg".to_owned(), "ahrhger".to_owned()],
            finished: false,
            show: false,
            dialogues: json::parse(&std::fs::read_to_string("res/dialogues.json").unwrap())
                .unwrap(),
        }
    }
}

pub fn apply_word_wrap_to_dialog(render: &RenderingState, dialog: &mut DialogState) {
    if dialog.text.is_empty() {
        return;
    };
    let (canvas_w, canvas_h) = render.canvas.logical_size();
    let margin = 5u32;
    let height = 70u32;
    let font_size = 8.0;
    let settings = LayoutSettings {
        x: (margin + margin) as f32,
        y: (canvas_h - margin + margin - height) as f32,
        max_width: Some((canvas_w - 4 * margin) as f32),
        ..LayoutSettings::default()
    };
    dialog.layout.reset(&settings);
    dialog.layout.append(
        render.fonts.as_slice(),
        &TextStyle::with_user_data(&dialog.text, font_size, dialog.font, Color::WHITE),
    );
    let mut new_text = dialog.text.clone();
    let mut new_line_places: Vec<_> = dialog
        .layout
        .lines()
        .unwrap()
        .iter()
        .map(|e| e.glyph_end)
        .collect();
    new_line_places.pop();
    for place in new_line_places.iter() {
        new_text = replace_nth_char(&new_text, *place, '\n');
    }
    dialog.text = new_text;
    fn replace_nth_char(s: &str, idx: usize, newchar: char) -> String {
        s.chars()
            .enumerate()
            .map(|(i, c)| if i == idx { newchar } else { c })
            .collect()
    }
}

pub fn update_dialog(
    dialog: &mut DialogState,
    input: &InputState,
    player: &mut PlayerState,
    render: &RenderingState,
) {
    dialog.show = true;
    let wants_to_continue = get_key_pressed(Keycode::Z, input);
    let wants_to_skip = get_key_pressed(Keycode::X, input);

    if dialog.text.is_empty() && !dialog.texts.is_empty() {
        dialog.finished = false;
        dialog.text = dialog.texts.first().unwrap().clone();
        dialog.texts.remove(0);
        apply_word_wrap_to_dialog(render, dialog);
    }
    if wants_to_skip {
        dialog.current_char = 999;
        dialog.finished = true;
    }
    if dialog.texts.is_empty() && wants_to_continue && dialog.finished {
        dialog.finished = true;
        dialog.show = false;
        player.state = PlayerStateMachine::Idling;
        player.wants_to_interact = false;
        dialog.current_char = 0;
        dialog.text.clear();
    } else if wants_to_continue && dialog.finished {
        dialog.finished = false;
        dialog.text = dialog.texts.first().unwrap().clone();
        dialog.texts.remove(0);
        dialog.current_char = 0;
        apply_word_wrap_to_dialog(render, dialog);
    }
}

pub fn set_dialog_from_id(id: u32, dialog: &mut DialogState, lang: &I18n) {
    match dialog.dialogues.clone() {
        JsonValue::Object(whole) => {
            let objj = whole.get(&id.to_string()).unwrap();
            match objj {
                JsonValue::Object(obj) => {
                    let _dialog_type = obj.get("type").unwrap().as_str().unwrap();
                    let texts_array = if let JsonValue::Array(texts) = obj.get("texts").unwrap() {
                        texts
                    } else {
                        panic!()
                    };
                    let texts: Vec<String> = texts_array
                        .iter()
                        .map(|text| {
                            if let JsonValue::Short(texte) = text {
                                texte.as_str()
                            } else {
                                panic!()
                            }
                        })
                        .map(|v| lang.t(v).as_str().unwrap().to_owned())
                        .collect();

                    dialog.texts = texts;
                    return;
                }
                _ => {}
            }
        }
        _ => {}
    }
    panic!("Failed to parse dialogues.json");
}

pub struct InputState {
    pub event_pump: EventPump,
    pub should_quit: bool,
    pub key_pressed_state: HashMap<Keycode, bool>,
    pub key_released_state: HashMap<Keycode, bool>,
    pub key_state: HashMap<Keycode, bool>,
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
    let mut dialog_state = DialogState::new();

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
        if player_state.state == PlayerStateMachine::Talking {
            update_dialog(
                &mut dialog_state,
                &input_state,
                &mut player_state,
                &rendering_state,
            );
        } else {
            move_player(&mut player_state, &input_state);
        }
        animate(&mut animation_state, &mut player_state, &mut enemies_state);
        count_dt(&mut physics_state);
        player_physics(&physics_state, &mut player_state);
        enemies_physics(&physics_state, &mut enemies_state);
        player_collision_interactables(&mut physics_state, &mut player_state);
        player_enemies_hit(&mut player_state, &mut enemies_state);

        //println!("STATE: {:?}", player_state.state);

        render(
            &mut rendering_state,
            &mut lang,
            &mut start_map,
            &render_player_state,
            &render_enemies_state,
            &render_physics_state,
            &mut dialog_state,
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
            InteractionResult::Inspect(inspect_id) => {
                set_dialog_from_id(inspect_id, &mut dialog_state, &lang)
            }
        }

        if player_state.state == PlayerStateMachine::Dying{
            start_map = switch_map(&mut loader, player_state.current_map.clone().as_str(), player_state.spawn_point.clone(), &lang, &mut rendering_state, &mut player_state, &mut enemies_state, &mut physics_state);
            player_state.state = PlayerStateMachine::Idling;
        }

        let _frame_end_time = frame_timer.elapsed();
        //println!("{}", 1.0/_frame_end_time.as_secs_f64());
        std::thread::sleep(std::time::Duration::from_millis(16)); // ONLY FOR DEV PURPOSES
        if input_state.should_quit {
            break;
        };
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
    player.spawn_point = spawn_number;
    player.current_map = path.to_owned();

    render.text_hints.clear();
    physics.colliders.clear();
    physics.interactables.clear();
    enemies.enemies.clear();

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
    if wanna_move {
        if player.state == PlayerStateMachine::Idling {
            player.state = PlayerStateMachine::Walking;
        }
    } else {
        if player.state == PlayerStateMachine::Walking {
            player.state = PlayerStateMachine::Idling;
        }
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
