extern crate sdl2;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod render;
use crate::render::*;

struct InputState{
    event_pump: EventPump, 
    should_quit: bool,
}


pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Project tomato", 1920, 1080)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(320, 180).map_err(|e| e.to_string())?;
    let event_pump = sdl_context.event_pump()?;

    let mut rendering_state = RenderingState::new(canvas);
    let mut input_state = InputState{event_pump, should_quit:false};

    let _pool = threadpool::ThreadPool::new(num_cpus::get());

    let mut loader = tiled::Loader::new();
    let mut start_map = loader.load_tmx_map("res/startmenu.tmx").unwrap();
    load_tilemap_to_textures(&mut rendering_state, &mut start_map);
    loop{
        let frame_timer = std::time::Instant::now();

        input(&mut input_state);
        if input_state.should_quit { break };

        render(&mut rendering_state, &mut start_map);
        
        let frame_end_time = frame_timer.elapsed();
        println!("{}", 1.0/frame_end_time.as_secs_f64());
    }

    Ok(())
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