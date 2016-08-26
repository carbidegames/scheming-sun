use std::time::{Duration, SystemTime};

pub struct FrameCounter {
    last: SystemTime,
    frame_counter: i32,
}

impl FrameCounter {
    pub fn new() -> Self {
        FrameCounter {
            last: SystemTime::now(),
            frame_counter: 0,
        }
    }

    pub fn tick(&mut self) {
        self.frame_counter += 1;
        let now = SystemTime::now();
        if now.duration_since(self.last).unwrap() >= Duration::new(1, 0) {
            println!("FPS: {}", self.frame_counter);
            self.frame_counter = 0;
            self.last = now;
        }
    }
}
