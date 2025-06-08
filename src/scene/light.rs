//! Necessary values to light up the scene.

use glam::{DVec3, U8Vec3};

/// The light object.
pub struct Light {
    /// Strength of the light that determines how much the color of the
    /// object will be illuminated. 0 means no light.
    pub strength: f64,
    /// The color of the light.
    pub color: U8Vec3,
    /// What type of light it is.
    pub light_type: LightType,
}
impl Light {
    /// Creates a light.
    pub fn new(strength: f64, color: U8Vec3, light_type: LightType) -> Self {
        match light_type {
            // Normalize direction for lights of type `AtInfinity`.
            LightType::AtInfinity(mut dir) => {
                // Ensure direction is normalizable.
                if dir.length() == 0.0 {
                    dir = DVec3::Z;
                }
                Light {
                    strength: strength.max(0.0),
                    color,
                    light_type: LightType::AtInfinity(dir.normalize()),
                }
            }
            // Ensure Strength is positive.
            LightType::Point {
                position,
                constant,
                linear,
                quadratic,
            } => Light {
                strength: strength.max(0.0),
                color,
                light_type: LightType::Point {
                    position,
                    constant: constant.max(0.0),
                    linear,
                    quadratic,
                },
            },
        }
    }
}
/// How the scene will be light up.
pub enum LightType {
    /// Light at infinity.
    ///
    /// * `DVec3` - The direction of the light.
    AtInfinity(DVec3),
    /// Light at point in space.
    ///
    /// * `position` - Position of the light in space.
    /// * `constant` - Light intensity will never dip below this value.
    /// * `linear` - How much the light intensity lowers linearly.
    /// * `quadratic` - How much the light intensity lowers qudratically.
    ///
    ///
    /// # Example usage
    /// ```
    /// let d = (position - vertex.position).length();
    /// let att = 1.0 / (constant + linear * d + quadratic * d * d);
    /// let intensity = (light.strength * att).clamp(0.0, 1.0);
    /// ```
    Point {
        /// Position of the point light.
        position: DVec3,
        /// Constant light attenuation value. Must be bigger than 0.
        constant: f32,
        /// Linear light attenuation value.
        linear: f32,
        /// Quadratic light attenuation value.
        quadratic: f32,
    },
}
