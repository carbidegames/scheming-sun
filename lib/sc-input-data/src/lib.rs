pub struct InputState {
    buttons: [bool; 4]
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            buttons: [false; 4]
        }
    }

    pub fn set(&mut self, button: Button, state: bool) {
        self.buttons[button as usize] = state;
    }

    pub fn get(&self, button: Button) -> bool {
        self.buttons[button as usize]
    }
}

pub enum Button {
    MoveForward,
    MoveBackward,
    MoveRight,
    MoveLeft,
    __DoNotMatch,
}

#[cfg(test)]
mod tests {
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
}
