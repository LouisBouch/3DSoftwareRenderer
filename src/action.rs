//! Container for anything action related. These actions are used to act on the scene.

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
// Rotates the camera to the right.
//
// * `f64` - Amount to Rotate (Rads).
// LookRight(f64),

