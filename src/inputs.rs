//! Handles the input from the user.
use crate::action::Action;
use glam::DVec2;
use std::collections::HashMap;
use winit::{self, keyboard::KeyCode};

/// Enum for the possible input states
pub enum InputState {
    /// Represents the key being pressed. (Activates both Pressed and Held actions)
    Pressed,
    /// Represents the key being held. (Activates held actions)
    Held,
    /// Represents the key being released. (Activates Released actions)
    Released,
    /// This state occurs when the key was released before the pressed key was consumed.
    /// (Activates Pressed, Held, and Released actions)
    PressedReleased,
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
    /// Converts hardware mouse changes into camera rotation.
    sensitivity: f32,
    /// Change in mouse position since last time the inputs were checked.
    /// None if no changes.
    mouse_delta: Option<DVec2>,
    // mouse_button_states: HashMap<KeyCode, InputState>,
    // 3 more hashmaps
}
impl InputHandler {
    /// Creates a new input state, which will store the actions of keypresses
    /// and their state (held or not).
    pub fn new() -> InputHandler {
        let mut input_handler = InputHandler {
            key_states: HashMap::new(),
            pressed_action: HashMap::new(),
            held_action: HashMap::new(),
            released_action: HashMap::new(),
            sensitivity: 1.0,
            mouse_delta: None,
        };
        input_handler.setup_default_bindings();
        input_handler
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
    pub fn bind(&mut self, input_state: InputState, key_code: KeyCode, action: Action) {
        match input_state {
            InputState::Pressed => {
                self.pressed_action.insert(key_code, action);
            }
            InputState::Held => {
                self.held_action.insert(key_code, action);
            }
            InputState::Released => {
                self.released_action.insert(key_code, action);
            }
            _ => {}
        };
    }
    /// Collects the list of actions that need to be executed.
    ///
    /// Given the key states and their associated actions, create a list of actions that will need
    /// to be executed. After being collected, keys with state `InputState::Released` are removed
    /// from the set, while keys with state `InputState::Pressed` are elevated to `InputState::Held`.
    ///
    /// # Return
    ///
    /// The list of actions that need to be acted upon.
    pub fn collect_actions(&mut self) -> Vec<Action> {
        let mut actions = Vec::new();
        let mut key_to_delete = Vec::new();
        for (k, v) in self.key_states.iter_mut() {
            match v {
                InputState::Held => {
                    // Check if a held action exists for the key.
                    if let Some(action) = self.held_action.get(k) {
                        actions.push(action.clone());
                    }
                }
                InputState::Released => {
                    // Check if a released action exists for the key.
                    if let Some(action) = self.released_action.get(k) {
                        actions.push(action.clone());
                    }
                    // Now remove the key from the list given that it has been released.
                    key_to_delete.push(k.clone());
                }
                InputState::Pressed => {
                    // First, check if a pressed action exists for the key.
                    if let Some(action) = self.pressed_action.get(k) {
                        actions.push(action.clone());
                    } else {
                        // If not, check if a held action exists for the key.
                        if let Some(action) = self.held_action.get(k) {
                            actions.push(action.clone());
                        }
                    }
                    // After registering the pressed key, it goes from pressed to held. That way,
                    // further readings do not mistake the key being pressed down twice.
                    *v = InputState::Held;
                }
                InputState::PressedReleased => {
                    // First check if pressed action exists for the key.
                    if let Some(action) = self.pressed_action.get(k) {
                        actions.push(action.clone());
                    } else {
                        // If not, check if a held action exists for the key.
                        if let Some(action) = self.held_action.get(k) {
                            actions.push(action.clone());
                        }
                    }
                    // Given that the key was pressed AND released in a single
                    // frame, check if a released action exists for the key.
                    if let Some(action) = self.released_action.get(k) {
                        actions.push(action.clone());
                    }
                    // Now remove the key from the list given that it has been released.
                    key_to_delete.push(k.clone());
                }
            }
        }
        // Delete from the list the keys that were released.
        for key in key_to_delete.iter() {
            self.key_states.remove(key);
        }
        // Collect mouse movements.
        let shift_pressed = self.key_states.get(&KeyCode::ShiftLeft).is_some();
        if let Some(DVec2 { x, y }) = self.mouse_delta.as_ref() {
            // Roll instead when shfit is pressed.
            if !shift_pressed {
                actions.push(Action::RotateCamera {
                    yaw: -x * self.sensitivity as f64,
                    pitch: -y * self.sensitivity as f64,
                    roll: 0.0,
                });
            } else {
                actions.push(Action::RotateCamera {
                    yaw: 0.0,
                    pitch: 0.0,
                    roll: -x * self.sensitivity as f64,
                });
            }
            // Now that the action was prepared, reset the delta.
            self.mouse_delta = None;
        }
        return actions;
    }
    /// Adds a key to the list after it is pressed.
    ///
    /// This method is called when a key is pressed, which adds it to the list of pressed keys.
    ///
    /// # Arguments
    ///
    /// * `key_code` - The code of the key that is pressed.
    pub fn press_key(&mut self, key_code: KeyCode) {
        if !self.key_states.contains_key(&key_code) {
            self.key_states.insert(key_code, InputState::Pressed);
            println!("key {:?} was pressed", key_code);
            // std::thread::sleep(time::Duration::from_millis(500));
        }
    }
    /// Updates key to released state.
    ///
    /// This method is called when a key is released, which sets its input state to released.
    ///
    /// # Arguments
    ///
    /// * `key_code` - The code of the key that was released.
    pub fn release_key(&mut self, key_code: KeyCode) {
        let Some(state) = self.key_states.get_mut(&key_code) else {
            println!("Key {:?} was released without being pressed.", key_code);
            return;
        };
        match state {
            InputState::Pressed => {
                *state = InputState::PressedReleased;
            }
            InputState::Held => {
                *state = InputState::Released;
            }
            _ => {}
        }
        println!("key {:?} was released", key_code);
    }
    /// Updates the mouse delta when the mouse is moved.
    ///
    /// This method is called when the mouse is moved, which adds the hardware detected movement to
    /// the total movement since the last input collection.
    ///
    /// # Arguments
    ///
    /// * `delta` - The raw mouse input detected by the hardware.
    pub fn mouse_move_raw(&mut self, new_delta: &DVec2) {
        // Add the delta if there is a delta.
        if let Some(current_delta) = self.mouse_delta.as_mut() {
            *current_delta += new_delta;
        }
        // Otherwise set the total mouse delta to be the current delta.
        else {
            self.mouse_delta = Some(*new_delta);
        }
    }
    /// Creates the default bindings.
    ///
    /// Default bindings include movement bindings, speed increases, etc.
    fn setup_default_bindings(&mut self) {
        // Setup basic movement bindings.
        self.held_action.insert(KeyCode::KeyW, Action::MoveForwards);
        self.held_action.insert(KeyCode::KeyA, Action::MoveLeft);
        self.held_action
            .insert(KeyCode::KeyS, Action::MoveBackwards);
        self.held_action.insert(KeyCode::KeyD, Action::MoveRight);
        self.held_action.insert(KeyCode::Space, Action::MoveUp);
        self.held_action
            .insert(KeyCode::ControlLeft, Action::MoveDown);
    }
}
