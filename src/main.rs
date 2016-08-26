extern crate sc_frontend;

use sc_frontend::{Frontend, FrontendEvent};

fn main() {
    let mut frontend = Frontend::start();

    'gameloop: loop {
        // Handle frontend events sent to us
        while let Some(event) = frontend.try_recv() {
            match event {
                FrontendEvent::Closed => break 'gameloop,
                _ => {}
            }
        }
    }

    frontend.stop();
}
