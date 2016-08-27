use std::time::{Duration, Instant};

pub struct FrameCounter {
    last: Instant,
    last_delta: f32,

    last_fps_logged: Instant,
    frame_counter: i32,
}

impl FrameCounter {
    pub fn new() -> Self {
        let now = Instant::now();

        FrameCounter {
            last: now,
            last_delta: 0.016,

            last_fps_logged: now,
            frame_counter: 0,
        }
    }

    pub fn tick(&mut self) {
        // Increment the counter for the FPS logging and store the elapsed delta
        self.frame_counter += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last);
        self.last_delta = (elapsed.as_secs() as f32) + (elapsed.subsec_nanos() as f32 / 1_000_000_000.0);
        self.last = now;

        // Keep track of the FPS and log it every second
        let elapsed_since_logged = now.duration_since(self.last_fps_logged);
        if elapsed_since_logged >= Duration::new(1, 0) {
            println!("FPS: {}", self.frame_counter);
            self.frame_counter = 0;
            self.last_fps_logged += elapsed_since_logged;
        }
    }

    pub fn delta(&self) -> f32 {
        self.last_delta
    }
}
