use std::sync::mpsc::{Sender, Receiver};
use sc_input_data::{Button, InputState};
use {ClientGameEvent, ClientGameCommand};

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

pub fn run(events: Receiver<ClientGameEvent>, commands: Sender<ClientGameCommand>) {
    let mut input = InputState::new();
    let mut world = WorldState::new();

    'gameloop: loop {
        // Handle frontend events sent to us
        while let Ok(event) = events.try_recv() {
            match event {
                ClientGameEvent::Closed => break 'gameloop,
                ClientGameEvent::ButtonState(button, state) => input.set(button, state),
                _ => {}
            }
        }

        // Update the world
        world.update(&input);

        // Sleep just a bit, don't keep the CPU busy constantly
        // TODO: Do this in a smarter way, this can way overshoot on some CPUs
        ::std::thread::sleep(::std::time::Duration::from_millis(10));
    }

    commands.send(ClientGameCommand::Stop).unwrap();
}
