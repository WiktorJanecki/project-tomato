use glam::Vec2;
use r_i18n::I18n;

use crate::{
    enemy::EnemiesState,
    physics::{Collider, Interactions, PhysicsState},
    render::{RenderingState, TilemapState},
    switch_map,
};

#[derive(Clone, PartialEq, Debug)]
pub enum PlayerStateMachine {
    Idling,
    Walking,
    Falling,
    Talking,
    Dying,
}

#[derive(Clone)]
pub struct PlayerState {
    pub _player_sprite_path: &'static str,
    pub x: f32,
    pub y: f32,
    pub width: u32,
    pub height: u32,

    pub hitbox: Collider,

    pub added_velocity: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,

    pub wants_dir: f32,
    pub wants_to_jump: bool,
    pub wants_to_interact: bool,

    pub state: PlayerStateMachine,
    pub is_grounded: bool,
    pub is_sliding: bool,
    pub coyote_time_counter: f32,
    pub jump_buffer_counter: f32,
    pub can_interact: bool,
    pub spawn_point: u32,
    pub current_map: String,
}

impl PlayerState {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            _player_sprite_path: "res/player.png",
            width: 8,
            height: 16,
            x: x,
            y: y,
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
            is_sliding: false,
            coyote_time_counter: 0.0,
            jump_buffer_counter: 0.0,
            wants_to_interact: false,
            can_interact: false,
            state: PlayerStateMachine::Idling,
            spawn_point: 0,
            current_map: "".to_owned(),
        }
    }
}

pub fn load_player_spawn(player: &mut PlayerState, tile: &TilemapState, spawn: u32) {
    for layer in tile.layers() {
        if layer.name == "PlayerSpawners" {
            match layer.layer_type() {
                tiled::LayerType::TileLayer(_) => {}
                tiled::LayerType::ObjectLayer(obj) => {
                    for o in obj.objects() {
                        if let Some(spawn_place_enum) = o.properties.get("spawn place") {
                            match spawn_place_enum {
                                tiled::PropertyValue::IntValue(x) => {
                                    if *x == spawn as i32 {
                                        player.x = o.x;
                                        player.y = o.y - player.height as f32; // obj origin is bottom left in tiled whereas top left in sdl
                                        return;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                tiled::LayerType::ImageLayer(_) => {}
                tiled::LayerType::GroupLayer(_) => {}
            }
        }
    }
    panic!("PLAYER SPAWN NOT FOUND");
}

pub enum InteractionResult {
    Nothing,
    ChangeMap(TilemapState),
    Inspect(u32),
}

pub fn player_interact(
    loader: &mut tiled::Loader,
    lang: &I18n,
    render: &mut RenderingState,
    player: &mut PlayerState,
    enemies: &mut EnemiesState,
    physics: &mut PhysicsState,
) -> InteractionResult {
    if player.can_interact
        && player.wants_to_interact
        && (player.state == PlayerStateMachine::Idling
            || player.state == PlayerStateMachine::Walking)
    {
        let mut interactable = None;
        for int in physics.interactables.iter() {
            if int.is_in_collider {
                interactable = Some(int.clone());
                break;
            }
        }
        if interactable.is_none() {
            return InteractionResult::Nothing;
        }
        match interactable.unwrap().interaction {
            Interactions::ChangeMap(path, numb) => {
                return InteractionResult::ChangeMap(switch_map(
                    loader, &path, numb, lang, render, player, enemies, physics,
                ));
            }
            Interactions::Inspect(inspect_id) => {
                player.state = PlayerStateMachine::Talking;
                return InteractionResult::Inspect(inspect_id);
            }
        }
    }
    return InteractionResult::Nothing;
}
