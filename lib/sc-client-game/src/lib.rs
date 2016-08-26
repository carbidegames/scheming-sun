extern crate sc_input_data;

mod runtime;

use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};
use sc_input_data::Button;

pub struct ClientGame {
    _handle: JoinHandle<()>,

    event_sender: Sender<ClientGameEvent>,
    command_receiver: Receiver<ClientGameCommand>,
}

impl ClientGame {
    pub fn init() -> Self {
        let (event_s, event_r) = mpsc::channel();
        let (command_s, command_r) = mpsc::channel();

        let handle = thread::spawn(move || {
            runtime::run(event_r, command_s);
        });

        ClientGame {
            _handle: handle,

            event_sender: event_s,
            command_receiver: command_r,
        }
    }

    pub fn send_event(&mut self, event: ClientGameEvent) {
        self.event_sender.send(event).unwrap();
    }

    pub fn next_command(&mut self) -> Option<ClientGameCommand> {
        self.command_receiver.try_recv().ok()
    }
}

pub enum ClientGameEvent {
    Closed,
    ButtonState(Button, bool),
    __DoNotMatch,
}

pub enum ClientGameCommand {
    Stop,
}
