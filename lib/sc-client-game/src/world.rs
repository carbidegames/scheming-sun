use sc_input_data::{Button, InputState};
use camera::Camera;

pub struct ClientWorld {
    camera: Camera,
    teapot: f32,
}

impl ClientWorld {
    pub fn new() -> Self {
        ClientWorld {
            camera: Camera::new(),
            teapot: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32, input: &InputState) {
        if input.get(Button::MoveForward) {
            self.teapot += 1.0 * delta;
        }
        if input.get(Button::MoveBackward) {
            self.teapot -= 1.0 * delta;
        }

        self.camera.update(delta, input);
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn teapot(&self) -> f32 {
        self.teapot
    }
}
