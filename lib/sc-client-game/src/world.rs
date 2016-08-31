use sc_input_data::InputState;
use camera::Camera;

pub struct ClientWorld {
    camera: Camera,
}

impl ClientWorld {
    pub fn new() -> Self {
        ClientWorld {
            camera: Camera::new(),
        }
    }

    pub fn update(&mut self, delta: f32, input: &InputState) {
        self.camera.update(delta, input);
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}
