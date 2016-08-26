extern crate sc_input_data;

use std::collections::VecDeque;
use sc_input_data::{Button, InputState};

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
            ClientGameEvent::Closed => self.commands.push_back(ClientGameCommand::Stop),
            ClientGameEvent::ButtonState(button, state) => self.input.set(button, state),
            _ => {}
        }
    }

    pub fn next_command(&mut self) -> Option<ClientGameCommand> {
        self.commands.pop_front()
    }

    pub fn update(&mut self) {
        // Update the world
        self.world.update(&self.input);
    }
}

pub struct ClientWorld {
    teapot: f32
}

impl ClientWorld {
    fn new() -> Self {
        ClientWorld {
            teapot: 0.0
        }
    }

    fn update(&mut self, input: &InputState) {
        if input.get(Button::MoveForward) {
            self.teapot += 0.01;
        }
        if input.get(Button::MoveBackward) {
            self.teapot -= 0.01;
        }
    }

    pub fn teapot(&self) -> f32 {
        self.teapot
    }
}

pub enum ClientGameEvent {
    Closed,
    ButtonState(Button, bool),
    __DoNotMatch,
}

pub enum ClientGameCommand {
    Stop,
    __DoNotMatch,
}
