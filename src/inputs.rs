//! Handles the input from the user.
use std::collections::{HashMap, HashSet};
use winit;

/// Conatins a list of possible actions.
pub enum Action {
    /// Makes the camera move forward.
    MoveForwards,
    /// Makes the camera move backward.
    MoveBackwards,
    /// Makes the camera move left.
    MoveLeft,
    /// Makes the camera move Right.
    MoveRight,

    /// Rotates the camera upwards.
    LookUp,
    /// Rotates the camera downwards.
    LookDown,
    /// Rotates the camera to the left.
    LookLeft,
    /// Rotates the camera to the right.
    LookRight,
}

/// Handles the user inputs.
pub struct InputState {
    /// List of keys that are currently being held down.
    held_keys: HashSet<winit::event::KeyEvent>,
    /// List of action for each key when it is pressed.
    pressed_action: HashMap<winit::event::KeyEvent, Action>,
    /// List of action for each key when it is held.
    held_action: HashMap<winit::event::KeyEvent, Action>,
    /// List of action for each key when it is released.
    released_action: HashMap<winit::event::KeyEvent, Action>,
}
impl InputState {
    /// Creates a new input state, which will store the actions of keypresses
    /// and their state (held or not).
    pub fn new() -> InputState {
        InputState {
            held_keys: HashSet::new(),
            pressed_action: HashMap::new(),
            held_action: HashMap::new(),
            released_action: HashMap::new(),
        }
    }
}
