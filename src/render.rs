use std::collections::HashMap;

use fontdue::layout::Layout;
use fontdue::layout::LayoutSettings;
use fontdue::layout::TextStyle;
use fontdue::Font;
use fontdue_sdl2::FontTexture;
use r_i18n::I18n;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use tiled::ObjectShape;
use tiled::PropertyValue;

use crate::DialogState;
use crate::EnemiesState;
use crate::PhysicsState;
use crate::PlayerState;

pub type TilemapState = tiled::Map;

pub struct RenderingState {
    pub canvas: Canvas<Window>,
    pub camera: sdl2::rect::Rect,
    pub texture_creator: TextureCreator<WindowContext>,
    pub textures: HashMap<String, Texture>,
    pub font_texture: FontTexture,
    pub fonts: Vec<Font>,

    pub text_hints: Vec<Layout<Color>>,
}
#[derive(Clone)]
pub struct AnimationFrame{
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone)]
pub struct Animation{
    pub frames: Vec<AnimationFrame>,
    pub time_per_frame: f32,
    pub timer: f32,
    pub current_frame: usize,
}

pub struct AnimationState{
    pub dt: f32,
    pub dt_timer: std::time::Instant,
}

impl AnimationState{
    pub fn new() -> Self{
        Self { dt: 0.0, dt_timer: std::time::Instant::now() }
    }
}

impl Animation{
    pub fn new(time_per_frame: f32) -> Self{
        Self{
            frames: vec![],
            timer: 0.0,
            time_per_frame,
            current_frame: 0,
        }
    }
}

pub fn animate(animation: &mut AnimationState, _player: &mut PlayerState, enemies: &mut EnemiesState){
    let now = std::time::Instant::now();
    let dt = (now.duration_since(animation.dt_timer).as_secs_f64()) as f32;
    animation.dt_timer = std::time::Instant::now();

    for enemy in enemies.enemies.iter_mut(){
        let anim = &mut enemy.animation;
        anim.timer += dt;
        if anim.timer > 0.0 {
            anim.timer = -anim.time_per_frame;
            anim.current_frame = (anim.current_frame + 1) %anim.frames.len();
        }
    }
}

impl RenderingState {
    pub fn new(canvas: Canvas<Window>) -> RenderingState {
        let texture_creator = canvas.texture_creator();
        let font_texture = FontTexture::new(&texture_creator).unwrap();
        let font = include_bytes!("../res/kongtext.ttf") as &[u8];
        let kongtext = Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let fonts = vec![kongtext];

        let mut x = RenderingState {
            canvas,
            camera: sdl2::rect::Rect::new(0, 0, 0, 0),
            texture_creator, // camera width = map width etc
            textures: HashMap::new(),
            font_texture: font_texture,
            fonts: fonts,

            text_hints: vec![],
        };

        let up_arrow_texture = x
            .texture_creator
            .load_texture("res/arrowglyph.png")
            .unwrap();
        x.textures
            .insert("res/arrowglyph.png".to_owned(), up_arrow_texture);
        x
    }
}

pub fn load_tilemap_to_textures(state: &mut RenderingState, tile_state: &TilemapState) {
    for tileset in tile_state.tilesets().to_owned() {
        let image = tileset.image.as_ref().unwrap();
        let path = image.source.as_os_str().to_str().unwrap();
        let txt = state.texture_creator.load_texture(path).unwrap();
        let name = tileset.name.clone();
        state.textures.insert(name, txt);
    }
    state
        .camera
        .set_width(tile_state.width * tile_state.tile_width);
    state
        .camera
        .set_height(tile_state.height * tile_state.tile_height);
}

pub fn load_tilemap_to_text_hints(state: &mut RenderingState, tile: &TilemapState, lang: &I18n) {
    for layer in tile.layers() {
        if layer.name == "TextHints" {
            match layer.layer_type() {
                tiled::LayerType::ObjectLayer(objl) => {
                    for obj in objl.objects() {
                        let font = if let PropertyValue::IntValue(font) =
                            obj.properties.get("font").unwrap()
                        {
                            font
                        } else {
                            panic!()
                        };
                        let text_untraslated = if let PropertyValue::StringValue(text) =
                            obj.properties.get("text").unwrap()
                        {
                            text
                        } else {
                            panic!()
                        };
                        let text = lang.t(&text_untraslated).as_str().unwrap();
                        let height = if let ObjectShape::Rect { height, .. } = obj.shape {
                            height
                        } else {
                            panic!()
                        };
                        let mut layout =
                            Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
                        let mut width = 0.0;
                        for chr in text.to_owned().chars() {
                            width += state
                                .fonts
                                .get(*font as usize)
                                .unwrap()
                                .metrics(chr, height)
                                .advance_width;
                        }
                        layout.reset(&LayoutSettings {
                            x: obj.x - width / 2.0,
                            y: obj.y,
                            ..Default::default()
                        });
                        layout.append(
                            state.fonts.as_slice(),
                            &TextStyle::with_user_data(text, height, *font as usize, Color::WHITE),
                        );
                        state.text_hints.push(layout);
                    }
                }
                _ => {}
            }
        }
    }
}

fn render_dialog(render: &mut RenderingState, dialog: &mut DialogState, lang: &I18n){
    let (canvas_w, canvas_h) = render.canvas.logical_size();

    let margin = 5u32;
    //let frame = 2u32;
    let height = 70u32;
    let bg = Rect::new((margin) as i32, (canvas_h - margin - height) as i32,canvas_w - margin - margin, height);
    render.canvas.set_draw_color(Color::BLACK);
    render.canvas.fill_rect(bg).unwrap();
    render.canvas.set_draw_color(dialog.color);
    render.canvas.draw_rect(bg).unwrap();
    let settings = LayoutSettings{ x: (margin + margin) as f32, y: (canvas_h-margin+margin-height) as f32, max_width: Some((canvas_w-4*margin) as f32), ..LayoutSettings::default() };
    dialog.layout.reset(&settings);
    let font_size = 8.0;
    dialog.current_char+=1;
    dialog.current_char = dialog.current_char.clamp(0, dialog.text.len());
    let text = &dialog.text[..dialog.current_char];
    dialog.layout.append(render.fonts.as_slice(), &TextStyle::with_user_data(text, font_size, dialog.font, Color::WHITE));
    if dialog.layout.lines().unwrap().last().unwrap().baseline_y > (canvas_h - margin) as f32{ // SOME ERROR HANDLING PLZ
        println!("PRZYPAŁ");
    }

    render.font_texture.draw_text(&mut render.canvas, &render.fonts, dialog.layout.glyphs()).unwrap();
}

fn render_tilemap(state: &mut RenderingState, tile_state: &TilemapState) {
    for layer in tile_state.layers() {
        match layer.layer_type() {
            tiled::LayerType::TileLayer(tile_layer) => {
                match tile_layer {
                    tiled::TileLayer::Finite(tiles) => {
                        let width = tiles.width(); // how many tiles in map
                        let height = tiles.height();
                        for i in 0..height {
                            for j in 0..width {
                                if let Some(tile) = tiles.get_tile(j as i32, i as i32) {
                                    let tile_prop = tile.get_tile().unwrap(); // get another fucking version of tile data
                                    let tilemap_id = tile_prop.tileset().name.clone(); // used for texture
                                    let txt = state.textures.get(&tilemap_id).unwrap(); // get texture

                                    let tile_width = tile_prop.tileset().tile_width; // width of tile in pixels
                                    let tile_height = tile_prop.tileset().tile_height;

                                    let dst = sdl2::rect::Rect::new(
                                        (j * tile_width) as i32 + state.camera.x,
                                        (i * tile_height) as i32 + state.camera.y,
                                        tile_width,
                                        tile_height,
                                    ); // where to render
                                    let x = tile.id() % (tile_prop.tileset().columns); // position in tileset
                                    let y = tile.id()
                                        / (tile_prop.tileset().tilecount
                                            / tile_prop.tileset().columns); // position in tileset
                                    let src = sdl2::rect::Rect::new(
                                        (x * tile_width) as i32,
                                        (y * tile_height) as i32,
                                        tile_width,
                                        tile_height,
                                    ); // get texture from "texture atlas"
                                    state
                                        .canvas
                                        .copy_ex(txt, src, dst, 0.0, None, tile.flip_h, tile.flip_v)
                                        .unwrap(); //render
                                }
                            }
                        }
                    }
                    tiled::TileLayer::Infinite(_) => {}
                }
            }
            tiled::LayerType::ObjectLayer(_) => {}
            tiled::LayerType::ImageLayer(_) => {}
            tiled::LayerType::GroupLayer(_) => {}
        }
    }
}

pub fn render_text_hints(state: &mut RenderingState) {
    for layout in state.text_hints.iter() {
        state
            .font_texture
            .draw_text_at(
                &mut state.canvas,
                &state.fonts,
                layout.glyphs(),
                state.camera.x,
                state.camera.y,
            )
            .unwrap();
    }
}

pub fn render(
    state: &mut RenderingState,
    lang: &mut I18n,
    tile_state: &TilemapState,
    player: &PlayerState,
    enemies: &EnemiesState,
    _physics: &PhysicsState,
    dialog: &mut DialogState,
) {
    state.canvas.set_draw_color(Color::RGB(0, 0, 0));
    state.canvas.clear();
    let (canvas_w, canvas_h) = state.canvas.logical_size();

    // center camera to player,
    // camera x and y are inverted (*-1) therefore every calculation must be inverted too
    state
        .camera
        .set_x(-(player.x as i32 + player.width as i32 / 2) + canvas_w as i32 / 2);
    state
        .camera
        .set_y(-(player.y as i32 + player.height as i32 / 2) + canvas_h as i32 / 2);

    //set camera bounds
    if state.camera.x > 0 {
        state.camera.set_x(0);
    }
    if state.camera.y > 0 {
        state.camera.set_y(0);
    }
    if (-state.camera.x + canvas_w as i32) > state.camera.width() as i32 {
        state
            .camera
            .set_x(state.camera.width() as i32 * -1 + canvas_w as i32);
    }
    if (-state.camera.y + canvas_h as i32) > state.camera.height() as i32 {
        state
            .camera
            .set_y(state.camera.height() as i32 * -1 + canvas_h as i32);
    }

    render_tilemap(state, tile_state);
    render_text_hints(state);
    render_enemies(state, enemies);
    
    // render player
    let dst = sdl2::rect::Rect::new(
        player.x as i32 + state.camera.x,
        player.y as i32 + state.camera.y,
        player.width,
        player.height,
    );
    state.canvas.set_draw_color(Color::RGB(255, 255, 0));
    state.canvas.fill_rect(dst).unwrap();

    if player.can_interact {
        let dst = sdl2::rect::Rect::new(
            player.x as i32 + state.camera.x + (0.5 * player.width as f32) as i32 - 4,
            player.y as i32 + state.camera.y - 16,
            8,
            8,
        );
        let txt = state.textures.get_mut("res/arrowglyph.png").unwrap();
        txt.set_color_mod(255, 255, 255);
        state.canvas.copy(txt, None, dst).unwrap();
    }
    
    //_render_colliders(state,player,_physics);
    render_dialog(state,dialog,lang);
    state.canvas.present();
}

pub fn render_enemies(state: &mut RenderingState, enemies: &EnemiesState){
    for enemy in enemies.enemies.iter(){
        let src = sdl2::rect::Rect::new(
            enemy.animation.frames[enemy.animation.current_frame].x,
            enemy.animation.frames[enemy.animation.current_frame].y,
            enemy.animation.frames[enemy.animation.current_frame].w,
            enemy.animation.frames[enemy.animation.current_frame].h,
        );
        let dst = sdl2::rect::Rect::new(
            enemy.x as i32+ state.camera.x,
            enemy.y as i32+ state.camera.y,
            enemy.width,
            enemy.height
        );
        let txt = state.textures.get(&enemy.texture_path).unwrap();
        state.canvas.copy(txt, src, dst).unwrap();
    }
}

pub fn _render_colliders(state: &mut RenderingState, player: &PlayerState, physics: &PhysicsState) {
    state.canvas.set_draw_color(Color::RGB(255, 0, 0));
    for col in physics.colliders.iter() {
        let rect = sdl2::rect::Rect::new(col.x, col.y, col.w, col.h);
        state.canvas.draw_rect(rect).unwrap();
    }
    let player_rect = sdl2::rect::Rect::new(
        player.x as i32 + player.hitbox.x + state.camera.x,
        player.y as i32 + player.hitbox.y + state.camera.y,
        player.hitbox.w,
        player.hitbox.h,
    );
    state.canvas.set_draw_color(Color::RGB(255, 0, 255));
    state.canvas.draw_rect(player_rect).unwrap();
}
