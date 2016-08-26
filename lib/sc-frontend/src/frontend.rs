use std::collections::VecDeque;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Sender, Receiver};

use runtime;
use {FrontendCommand, FrontendEvent};

pub struct Frontend {
    handle: JoinHandle<()>,
    sender: Sender<FrontendCommand>,
    receiver: Receiver<Vec<FrontendEvent>>,

    event_buffer: VecDeque<FrontendEvent>,
}

impl Frontend {
    pub fn start() -> Self {
        let (sender, r_receiver) = mpsc::channel();
        let (r_sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            runtime::frontend_runtime(r_sender, r_receiver);
        });

        Frontend {
            handle: handle,
            sender: sender,
            receiver: receiver,

            event_buffer: VecDeque::new(),
        }
    }

    pub fn stop(self) {
        self.sender.send(FrontendCommand::Stop).unwrap();
        self.handle.join().unwrap();
    }

    pub fn try_recv(&mut self) -> Option<FrontendEvent> {
        // Check if we have an event buffered
        if let Some(event) = self.event_buffer.pop_front() {
            Some(event)
        } else {
            // We don't check if there's events waiting for us
            if let Ok(events) = self.receiver.try_recv() {
                // We've got new events, add them to the queue and send over the first one
                for event in events {
                    self.event_buffer.push_back(event);
                }
                self.event_buffer.pop_front()
            } else {
                // Nope, we're all out
                None
            }
        }
    }
}
