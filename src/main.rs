extern crate sc_frontend;
extern crate sc_input_data;

use sc_frontend::{Frontend, FrontendEvent};
use sc_input_data::{Button, InputState};

struct WorldState {
    teapot: f32
}

impl WorldState {
    fn new() -> Self {
        WorldState {
            teapot: 0.0
        }
    }

    fn update(&mut self, input: &InputState) {
        if input.get(Button::MoveForward) {
            self.teapot += 0.01;
            println!("{}", self.teapot);
        }
    }
}

fn main() {
    let mut frontend = Frontend::start();
    let mut input = InputState::new();
    let mut world = WorldState::new();

    'gameloop: loop {
        // Handle frontend events sent to us
        while let Some(event) = frontend.try_recv() {
            match event {
                FrontendEvent::Closed => break 'gameloop,
                FrontendEvent::ButtonState(button, state) => input.set(button, state),
                _ => {}
            }
        }

        // Update the world
        world.update(&input);

        // Sleep just a bit, don't keep the CPU busy constantly
        // TODO: Do this in a smarter way, this can way overshoot on some CPUs
        ::std::thread::sleep(::std::time::Duration::from_millis(10));
    }

    frontend.stop();
}
