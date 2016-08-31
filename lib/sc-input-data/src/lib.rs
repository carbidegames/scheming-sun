extern crate cgmath;

use cgmath::Vector2;

pub struct InputState {
    buttons: [bool; 8],
    mouse_position: Vector2<i32>,
    frame_mouse: Vector2<i32>,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            buttons: [false; 8],
            mouse_position: Vector2::new(0, 0),
            frame_mouse: Vector2::new(0, 0),
        }
    }

    pub fn set(&mut self, button: Button, state: bool) {
        self.buttons[button as usize] = state;
    }

    pub fn get(&self, button: Button) -> bool {
        self.buttons[button as usize]
    }

    pub fn add_mouse(&mut self, value: Vector2<i32>) {
        // Find the new position, then apply it
        let new = self.mouse_position + value;
        self.set_mouse(new, true);
    }

    pub fn set_mouse(&mut self, position: Vector2<i32>, should_track: bool) {
        if should_track {
            // Find the difference in positions and add that to the difference tracker
            let difference = position - self.mouse_position;
            self.frame_mouse += difference;
        }

        // Store the new position
        self.mouse_position = position;
    }

    pub fn end_frame(&mut self) {
        self.frame_mouse = Vector2::new(0, 0);
    }

    pub fn frame_mouse(&self) -> Vector2<i32> {
        self.frame_mouse
    }
}

pub enum Button {
    MoveForward,
    MoveBackward,
    MoveRight,
    MoveLeft,
    Menu,
    __DoNotMatch,
}

#[cfg(test)]
mod tests {
    use cgmath::Vector2;
    use {InputState, Button};

    #[test]
    fn set_changes_get_result() {
        let mut input = InputState::new();

        input.set(Button::MoveForward, true);
        assert!(input.get(Button::MoveForward));
        assert!(!input.get(Button::MoveBackward));

        input.set(Button::MoveForward, false);
        input.set(Button::MoveBackward, true);
        assert!(!input.get(Button::MoveForward));
        assert!(input.get(Button::MoveBackward));
    }

    #[test]
    fn tracks_accumulated_mouse() {
        let mut input = InputState::new();

        input.add_mouse(Vector2::new(20, 20));
        assert_eq!(input.frame_mouse(), Vector2::new(20, 20));

        input.end_frame();
        input.add_mouse(Vector2::new(8, 10));
        input.add_mouse(Vector2::new(8,  5));
        assert_eq!(input.frame_mouse(), Vector2::new(16, 15));
    }
}
