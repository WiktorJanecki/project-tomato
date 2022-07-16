
use std::collections::HashMap;


use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::video::WindowContext;

type TilemapState = tiled::Map;
pub struct RenderingState{
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    textures: HashMap<String, Texture>,
}

impl RenderingState{
    pub fn new(canvas: Canvas<Window>) -> RenderingState{
        let texture_creator = canvas.texture_creator();
        RenderingState { canvas, texture_creator, textures: HashMap::new() }
    }
}


pub fn load_tilemap_to_textures(state: &mut RenderingState, tile_state :&TilemapState){
    for tileset in tile_state.tilesets().to_owned(){
        let image = tileset.image.as_ref().unwrap();
        let path = image.source.as_os_str().to_str().unwrap();
        let txt = state.texture_creator.load_texture(path).unwrap();
        let name = tileset.name.clone();
        state.textures.insert(name, txt);
    }
}

fn render_tilemap(state: &mut RenderingState, tile_state :&TilemapState){
    let (_, canvas_h) = state.canvas.logical_size();
    for layer in tile_state.layers(){
        match layer.layer_type(){
            tiled::LayerType::TileLayer(tile_layer) => {
                match tile_layer{
                    tiled::TileLayer::Finite(tiles) => {
                        let width = tiles.width();
                        let height = tiles.height();
                        for i in 0..height{
                            for j in 0..width{
                                if let Some(tile) = tiles.get_tile(j as i32,  (i as i32)){
                                    let x = tile.get_tile().unwrap();
                                    let tilemap_id = x.tileset().name.clone();
                                    let txt = state.textures.get(&tilemap_id).unwrap();

                                    let tile_width = x.tileset().tile_width;
                                    let tile_height = x.tileset().tile_height;

                                    let tile_y_converted = (canvas_h).abs_diff(tile_height * height) as i32 + (i*tile_height) as i32;
                                    let dst = sdl2::rect::Rect::new((j*tile_width) as i32, tile_y_converted,tile_width,tile_height);
                                    let x = tile.id() % (3); // 3 is tiles in tileset
                                    let y = tile.id() / 3;
                                    let src = sdl2::rect::Rect::new((x*tile_width) as i32, (y*tile_height) as i32, tile_width, tile_height);
                                    // TODO: BŁAGAM NAPRAW TO
                                    state.canvas.copy_ex(txt, src, dst, 0.0, None, tile.flip_h, tile.flip_v).unwrap();
                                }
                            }
                        }
                    },
                    tiled::TileLayer::Infinite(_) => {},
                }
            },
            tiled::LayerType::ObjectLayer(_) =>  {},
            tiled::LayerType::ImageLayer(_) =>  {},
            tiled::LayerType::GroupLayer(_) =>  {},
        }
    }
}


pub fn render(state: &mut RenderingState, tile_state :&TilemapState){
    state.canvas.set_draw_color(Color::RGB(0, 0, 0));
    state.canvas.clear();

    render_tilemap(state,tile_state);

    state.canvas.present();
}