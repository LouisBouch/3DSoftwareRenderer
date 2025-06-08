//! Contains the necessary APIs/values to shade geometry within a scene.

use glam::DVec3;

use crate::scene::light::Light;

/// Contains the values necessary to decdie which shader to use and how to use them.
pub struct Shader {
    /// The ambient lighting. A value between 0-1, where 0 means no ambient light and 1 means
    /// everything is fully illuminated.
    pub ambient: f64,
    /// The type of shader.
    pub shader_type: ShaderType,
}
impl Shader {
    /// Creates a shader.
    ///
    /// # Arguments:
    ///
    /// * `shader_type` - How the shading value is calculated.
    /// * `ambient` - The ambient lighting. A value between 0-1, where 0 means no ambient light and 1 means
    /// everything is fully illuminated.
    pub fn new(ambient: f64, shader_type: ShaderType) -> Self {
        Shader {
            ambient,
            shader_type,
        }
    }
    /// Defines how a shader will shade a pixel.
    ///
    /// # Arguments
    ///
    /// * `normal` - Normal of the surface the shader is currently working on (has to be normalized).
    /// * `lights` - List of lights populating the scene.
    ///
    /// # Return
    ///
    /// Value that dictates how the pixel is to be shaded.
    /// ```
    /// let shaded_color = color * shader.shade(...);
    /// ```
    pub fn shade(&self, normal: DVec3, lights: &[Light]) -> f64{
        let mut shading: f64 = self.ambient;
        for light in lights {
            match light.light_type {
                crate::scene::light::LightType::AtInfinity(dir) => {
                    shading += light.strength * normal.dot(-dir).max(0.0);
                },
                crate::scene::light::LightType::Point {..} => todo!("Implement point light shading"),
            }
        }
        shading.min(1.0)
    }
}
/// The different possible types of shaders.
pub enum ShaderType {
    /// Per-pixel shading.
    Phong,
    /// Interpolated shading from the triangles vertices.
    Gouraud,
    /// Single shading value per geometry face.
    Flat,
}
