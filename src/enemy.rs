use glam::Vec2;
use sdl2::image::LoadTexture;
use tiled::PropertyValue;

use crate::{physics::Collider, render::{RenderingState, TilemapState, Animation, AnimationFrame}};

#[derive(Clone)]
pub struct EnemiesState{
    pub  enemies: Vec<Enemy>,
}

impl EnemiesState {
    pub fn new() -> Self { Self { enemies:vec![] } }
}

#[derive(Clone)]
pub struct Enemy{
    pub x: f32,
    pub y: f32,
    pub width: u32,
    pub height: u32,
    pub velocity: Vec2,
    pub collider: Collider,
    pub dir: i32,
    pub texture_path: String,
    pub animation: Animation,
}

pub fn load_tilemap_to_enemies(enemies: &mut EnemiesState, tile_state: &TilemapState, render: &mut RenderingState) {
    for tileset in tile_state.tilesets().to_owned() {
        let image = tileset.image.as_ref().unwrap();
        let path = image.source.as_os_str().to_str().unwrap();
        let txt = render.texture_creator.load_texture(path).unwrap();
        let name = tileset.name.clone();
        render.textures.insert(name, txt);
    }
    let mut enemies_vec: Vec<Enemy> = vec![];
    for layer in tile_state.layers() {
        if layer.name == "Enemies" {
            match layer.layer_type() {
                tiled::LayerType::ObjectLayer(obj_layer) => {
                    for obj in obj_layer.objects() {
                        match obj.shape {
                            tiled::ObjectShape::Rect { width, height } => {
                                let x = obj.x as i32;
                                let y = obj.y as i32;
                                let w = width as u32;
                                let h = height as u32;
                                let col = Collider { y:0, x:0, w, h };
                                let dir = if let PropertyValue::IntValue(dir) = obj.properties.get("dir").unwrap() {dir} else {panic!()};
                                let txt = if let PropertyValue::StringValue(txt) = obj.properties.get("texture").unwrap() {txt} else {panic!()};
                                if !render.textures.contains_key(txt){
                                    let texture = render.texture_creator.load_texture(txt).unwrap();
                                    render.textures.insert(txt.to_owned(),texture);
                                }
                                // HARD CODED ANIMATIONS
                                let animation_time = 0.5;
                                if txt == "res/tomato_gumba.png" {
                                    let mut animation = Animation::new(animation_time);
                                    animation.frames.push(AnimationFrame{x: 0, y: 0, w: 16, h:16});
                                    animation.frames.push(AnimationFrame{x: 16, y: 0, w: 16, h:16});
                                    let enemy = Enemy{velocity: Vec2::ZERO,x: x as f32, y: y as f32, width: w, height: h, collider: col, dir: *dir, texture_path: txt.to_owned(), animation: animation};
                                    enemies_vec.push(enemy);
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
    enemies.enemies.append(&mut enemies_vec);
}
