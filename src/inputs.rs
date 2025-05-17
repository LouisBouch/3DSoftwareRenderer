//! Handles the input from the user.
use std::collections::HashMap;
use winit::{self, keyboard::KeyCode};
use crate::action::Action;

/// Enum for the possible input states
pub enum InputState {
    /// Represents the key being pressed.
    Pressed,
    /// Represents the key being held.
    Held,
    /// Represents the key being released.
    Released,
}

/// Handles the user inputs.
pub struct InputHandler {
    /// List of keys that are currently being pressed, held or released.
    key_states: HashMap<KeyCode, InputState>,
    /// List of action for each key when it is pressed.
    pressed_action: HashMap<KeyCode, Action>,
    /// List of action for each key when it is held.
    held_action: HashMap<KeyCode, Action>,
    /// List of action for each key when it is released.
    released_action: HashMap<KeyCode, Action>,
    // TODO: Include mouse events
    // last_mouse_pos: (f64, f64),
    // mouse_button_states: HashMap<KeyCode, InputState>,
    // 3 more hashmaps
}
impl InputHandler {
    /// Creates a new input state, which will store the actions of keypresses
    /// and their state (held or not).
    pub fn new() -> InputHandler {
        InputHandler {
            key_states: HashMap::new(),
            pressed_action: HashMap::new(),
            held_action: HashMap::new(),
            released_action: HashMap::new(),
        }
    }
    /// Binds a key.
    ///
    /// Creates a keybinding for a specific InputState and KeyCode.
    ///
    /// # Arguments
    ///
    /// * `input_state` - State of the key event.
    /// * `key_code` - Code of the key being pressed.
    /// * `action` - Action.
    fn bind(&mut self, input_state: InputState, key_code: KeyCode, action: Action) {
        match input_state {
            InputState::Pressed => self.pressed_action.insert(key_code, action),
            InputState::Held => self.held_action.insert(key_code, action),
            InputState::Released => self.released_action.insert(key_code, action),
        };
    }
    /// Collects the list of actions that need to be executed.
    ///
    /// Given the key states and their associated actions, create a list of actions that will need
    /// to be executed.
    ///
    /// # Return
    ///
    /// The list of actions that need to be acted upon.
    fn collect_actions(&mut self) -> Vec<Action> {
        todo!("Implement collect_actions method");
    }
    /// Adds a key to the list after it is pressed.
    ///
    /// This method is called when a key is pressed, which adds it to the list of pressed keys.
    ///
    /// # Arguments
    ///
    /// * `key_code` - The code of the key currently being pressed.
    fn press_key(&mut self, key_code: KeyCode) {
    }
    /// Updates key to released state.
    ///
    /// This method is called when a key is released, which sets its input state to released.
    ///
    /// # Arguments
    ///
    /// * `key_code` - The code of the key that was released.
    fn release_key(&mut self, key_code: KeyCode) {
    }
    /// Creates the default bindings.
    ///
    /// Default bindings include movement bindings, speed increases, etc.
    fn setup_default_bindings(&mut self) {
    }
}
