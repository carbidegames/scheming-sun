extern crate cgmath;
#[macro_use] extern crate vulkano;
extern crate vulkano_win;
extern crate winit;
extern crate sc_client_game;
extern crate sc_input_data;

mod framecounter;
mod frontend;
mod teapot;

mod vs { include!{concat!(env!("OUT_DIR"), "/shaders/src/shader_vs.glsl")} }
mod fs { include!{concat!(env!("OUT_DIR"), "/shaders/src/shader_fs.glsl")} }

use sc_client_game::{ClientGame};
use framecounter::FrameCounter;
use frontend::Frontend;

pub fn run() {
    let mut game = ClientGame::connect();
    let mut frontend = Frontend::init();
    let mut counter = FrameCounter::new();

    loop {
        // TODO: This needs a rework, the assumption that events just need to be sent over to the
        //  ClientGame is wrong, because the ClientGame only cares about information relevant to
        //  the world simulation.
        // Get the frontend events that have happened and send them over
        let keep_running = frontend.poll_events(|event| {
            game.handle_event(event);
        });
        if !keep_running { break; }

        // Update the backend
        game.update(counter.delta());

        // Check what the backend wants us to do
        if let Some(command) = game.next_command() {
            match command {
                _ => (),
            }
        }

        // Render the updated game state
        frontend.render(game.world());

        counter.tick();
    }
}
