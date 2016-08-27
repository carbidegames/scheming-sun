use cgmath::{Vector3, InnerSpace, Zero};
use sc_input_data::{InputState, Button};

pub struct Camera {
    position: Vector3<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vector3::new(0.0, 0.0, 2.0),
        }
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn update(&mut self, delta: f32, input: &InputState) {
        let mut direction = Vector3::zero();

        if input.get(Button::MoveForward) { direction.z -= 1.0; }
        if input.get(Button::MoveBackward) { direction.z += 1.0; }
        if input.get(Button::MoveRight) { direction.x += 1.0; }
        if input.get(Button::MoveLeft) { direction.x -= 1.0; }

        if direction != Vector3::zero() {
            self.position += direction.normalize() * delta;
        }
    }
}

#[cfg(test)]
mod tests {
    use sc_input_data::{InputState, Button};
    use Camera;

    #[test]
    fn moves_when_buttons_are_active() {
        let mut camera = Camera::new();
        let mut input = InputState::new();

        let initial = camera.position();

        input.set(Button::MoveForward, true);
        camera.update(0.1, &input);

        assert!(camera.position().z < initial.z);
    }
}
