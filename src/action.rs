//! Container for anything action related. These actions are used to act on the scene.

/// Conatins a list of possible actions.
#[derive(Clone)]
pub enum Action {
    /// Makes the camera move forward.
    MoveForwards,
    /// Makes the camera move backward.
    MoveBackwards,
    /// Makes the camera move left.
    MoveLeft,
    /// Makes the camera move right.
    MoveRight,
    /// Makes the camera move up.
    MoveUp,
    /// Makes the camera move down.
    MoveDown,

    /// Rotates the camera. (Usually yaw is applied first, but they can be combined in one smooth
    /// rotation through a weighted average of axis of rotation.)
    RotateCamera {
        /// Change in pitch/vertical angle (Rads).
        pitch: f64,
        /// Change in yaw/horizontal angle (Rads).
        yaw: f64,
    },
}
