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

use sc_client_game::{ClientGame, ClientGameEvent};
use sc_input_data::Button;
use framecounter::FrameCounter;
use frontend::{Frontend, FrontendEvent};

pub fn run() {
    let mut game = ClientGame::connect();
    let mut frontend = Frontend::init();
    let mut counter = FrameCounter::new();

    loop {
        // Get the frontend events and handle them or send them over
        let mut should_break = false;
        frontend.poll_events(|event| {
            match event {
                FrontendEvent::Close => should_break = true,
                FrontendEvent::ButtonState(Button::Menu, _) => should_break = true,
                FrontendEvent::ButtonState(b, s) =>
                    game.handle_event(ClientGameEvent::ButtonState(b, s)),
                FrontendEvent::MouseMove(position) =>
                    game.handle_event(ClientGameEvent::MouseMove(position)),
            }
        });
        if should_break { break; }

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
