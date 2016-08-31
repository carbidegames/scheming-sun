use cgmath::{Vector3, InnerSpace, Zero, Angle, Rad};
use sc_input_data::{InputState, Button};

pub struct Camera {
    position: Vector3<f32>,
    pitch: Rad<f32>,
    yaw: Rad<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vector3::new(0.0, 0.0, 2.0),
            pitch: Rad(0.0),
            yaw: Rad(0.0),
        }
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn pitch(&self) -> Rad<f32> {
        self.pitch
    }

    pub fn yaw(&self) -> Rad<f32> {
        self.yaw
    }

    pub fn update(&mut self, delta: f32, input: &InputState) {
        // Rotate to mouse input
        self.pitch -= Rad(input.frame_mouse().y as f32 * 0.0005);
        self.yaw -= Rad(input.frame_mouse().x as f32 * 0.0005);

        // Limit the pitch
        let lim = Rad::full_turn() * 0.25;
        if self.pitch > lim { self.pitch = lim; }
        if self.pitch < -lim { self.pitch = -lim; }

        // Move to button input
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
    use cgmath::{Vector2, Angle, Rad};
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

    #[test]
    fn mouse_input_changes_rotations() {
        let mut camera = Camera::new();
        let mut input = InputState::new();

        let initial_yaw = camera.yaw();
        let initial_pitch = camera.pitch();

        input.add_mouse(Vector2::new(20, 20));
        camera.update(0.1, &input);

        assert!(camera.yaw() != initial_yaw);
        assert!(camera.pitch() != initial_pitch);
    }

    #[test]
    fn pitch_is_limited() {
        let mut camera = Camera::new();
        let mut input = InputState::new();

        for _ in 0..100 {
            input.add_mouse(Vector2::new(0, 100));

            camera.update(0.1, &input);
            assert!(camera.pitch() < Rad::full_turn() * 0.251);
            assert!(camera.pitch() > Rad::full_turn() * -0.251);

            input.end_frame();
        }
    }
}
