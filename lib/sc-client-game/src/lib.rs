extern crate sc_input_data;

use std::collections::VecDeque;
use sc_input_data::{Button, InputState};

pub struct ClientGame {
    input: InputState,
    world: WorldState,

    commands: VecDeque<ClientGameCommand>,
}

impl ClientGame {
    pub fn connect() -> Self {
        ClientGame {
            input: InputState::new(),
            world: WorldState::new(),

            commands: VecDeque::new(),
        }
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

pub enum ClientGameEvent {
    Closed,
    ButtonState(Button, bool),
    __DoNotMatch,
}

pub enum ClientGameCommand {
    Stop,
    __DoNotMatch,
}
