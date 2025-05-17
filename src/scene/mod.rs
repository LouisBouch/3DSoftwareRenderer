//! Contains everything needed to render the environment.

use camera::Camera;

/// Contains everything in the scene that will be rendered.
pub struct Scene {
    /// The camera from which the environment will be seen.
    pub camera: Camera,
}
mod vertex;
mod mesh;
mod light;
pub mod camera;
