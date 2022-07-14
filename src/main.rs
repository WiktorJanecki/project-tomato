extern crate sdl2;

use std::sync::Mutex;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

struct RenderingState{
    canvas: Canvas<Window>,
}
struct InputState{
    event_pump: EventPump, 
    should_quit: bool,
}

struct TexturedRect<'a>{
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    txt: &'a sdl2::render::Texture<'a>,
}

struct TileState<'a>{
    rects: Vec<TexturedRect<'a>>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Project tomato", 800, 600)
        .position_centered()
        //.opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let event_pump = sdl_context.event_pump()?;

    let mut rendering_state = RenderingState{canvas};
    let mut input_state = InputState{event_pump, should_quit:false};

    let _pool = threadpool::ThreadPool::new(num_cpus::get());

    
    let surface = sdl2::surface::Surface::new(512, 512, sdl2::pixels::PixelFormatEnum::RGB24).unwrap();
    let texture = texture_creator.create_texture_from_surface(surface).unwrap();

    let rect = TexturedRect{x:0,y:0,w:512,h:512,txt:&texture};
    let rect2 = TexturedRect{x:522,y:0,w:512,h:512,txt:&texture};

    let tile_state = TileState{rects:vec![rect,rect2]};
    
    loop{
        input(&mut input_state);
        if input_state.should_quit { break };
        
        

        render(&mut rendering_state, &tile_state);
    }

    

    Ok(())
}

fn render(state: &mut RenderingState, tile_state: &TileState){
    let canvas = &mut state.canvas;
    canvas.set_draw_color(Color::RGB(255, 1, 1));
    canvas.clear();

    for rect in tile_state.rects.iter(){
        let r = sdl2::rect::Rect::new(rect.x,rect.y,rect.w,rect.h);
        canvas.copy(rect.txt, None, r);
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