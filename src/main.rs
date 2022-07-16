extern crate sdl2;

use std::collections::HashMap;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

struct RenderingState{
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}
struct InputState{
    event_pump: EventPump, 
    should_quit: bool,
}

type TilemapState = tiled::Map;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Project tomato", 1920,1080 )
        .position_centered()
        //.opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(320, 180).map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let event_pump = sdl_context.event_pump()?;

    let mut rendering_state = RenderingState{canvas, texture_creator};
    let mut input_state = InputState{event_pump, should_quit:false};

    let _pool = threadpool::ThreadPool::new(num_cpus::get());

    let mut loader = tiled::Loader::new();
    let mut start_map = loader.load_tmx_map("res/startmenu.tmx").unwrap();
    
    loop{
        input(&mut input_state);
        if input_state.should_quit { break };
        
        

        render(&mut rendering_state, &mut start_map);
    }

    Ok(())
}

fn render(state: &mut RenderingState, tile_state :&TilemapState){
    let canvas = &mut state.canvas;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // render tilemap
    let mut textures: HashMap<String, Texture> = HashMap::new();
    for tileset in tile_state.tilesets().to_owned(){
        let image = tileset.image.as_ref().unwrap();
        let path = image.source.as_os_str().to_str().unwrap();
        let txt = state.texture_creator.load_texture(path).unwrap();
        let name = tileset.name.clone();
        textures.insert(name, txt);
    }
    for layer in tile_state.layers(){
        match layer.layer_type(){
            tiled::LayerType::TileLayer(tile_layer) => {
                match tile_layer{
                    tiled::TileLayer::Finite(tiles) => {
                        let width = tiles.width();
                        let height = tiles.height();
                        for i in 0..height{
                            for j in 0..width{
                                if let Some(tile) = tiles.get_tile(j as i32, i as i32){
                                    let x = tile.get_tile().unwrap();
                                    let tilemap_id = x.tileset().name.clone();
                                    let txt = textures.get(&tilemap_id).unwrap();

                                    let tile_width = x.tileset().tile_width;
                                    let tile_height = x.tileset().tile_height;

                                    let dst = sdl2::rect::Rect::new((j*tile_width) as i32, (i*tile_height) as i32,tile_width,tile_height);
                                    let x = tile.id() % (3); // 3 is tiles in tileset
                                    let y = tile.id() / 3;
                                    let src = sdl2::rect::Rect::new((x*tile_width) as i32, (y*tile_height) as i32, tile_width, tile_height);
                                    // TODO: BŁAGAM NAPRAW TO
                                    canvas.copy(txt, src, dst).unwrap();
                                }
                            }
                        }
                    },
                    tiled::TileLayer::Infinite(_) => panic!("NOT IMPLEMENTED"),
                }
            },
            tiled::LayerType::ObjectLayer(_) =>  panic!("NOT IMPLEMENTED"),
            tiled::LayerType::ImageLayer(_) =>  panic!("NOT IMPLEMENTED"),
            tiled::LayerType::GroupLayer(_) =>  panic!("NOT IMPLEMENTED"),
        }
    }


    canvas.present();
}

fn input(state: &mut InputState){
    let event_pump = &mut state.event_pump;
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {state.should_quit = true; return;},
            _ => {}
        }
    }
}