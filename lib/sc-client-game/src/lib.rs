extern crate sc_input_data;
extern crate cgmath;

mod camera;
mod world;

use std::collections::VecDeque;
use sc_input_data::{Button, InputState};

pub use camera::Camera;
pub use world::ClientWorld;

pub struct ClientGame {
    input: InputState,
    world: ClientWorld,

    commands: VecDeque<ClientGameCommand>,
}

impl ClientGame {
    pub fn connect() -> Self {
        ClientGame {
            input: InputState::new(),
            world: ClientWorld::new(),

            commands: VecDeque::new(),
        }
    }

    pub fn world(&self) -> &ClientWorld {
        &self.world
    }

    pub fn handle_event(&mut self, event: ClientGameEvent) {
        match event {
            ClientGameEvent::ButtonState(button, state) => self.input.set(button, state),
            _ => {}
        }
    }

    pub fn next_command(&mut self) -> Option<ClientGameCommand> {
        self.commands.pop_front()
    }

    pub fn update(&mut self, delta: f32) {
        // Update the world
        self.world.update(delta, &self.input);

        self.input.end_frame();
    }
}

pub enum ClientGameEvent {
    ButtonState(Button, bool),
    __DoNotMatch,
}

pub enum ClientGameCommand {
    __DoNotMatch,
}
