//! Handles the loading of all ressources.

use crate::resources::texture::Texture;

/// Used to load default textures, textures from files or user defined textures.
pub struct TextureLoader {
    /// Sample the texture at different intervals. Bigger values will give worse quality textures.
    sampling: u32,
}
impl TextureLoader {
    /// Used to generate a default texture.
    ///
    /// Allows the user to specify a default texture and build it through this function.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The default pattern to use to create the texture.
    pub fn load_default_pattern(&mut self, pattern: DefaultPattern) -> Texture {
        match pattern {
            DefaultPattern::Checkered(size) => {
                todo!("Create pixels vector and use it to create Texture");
            },
        }
    }
}
/// A list of default patterns that can be used to quickly get a texture.
pub enum DefaultPattern {
    /// A black and white checkered pattern.
    ///
    /// - `u32` The size (in pixels) of the texture's primitve, which is a 2x2 square of black and
    /// white squares.
    Checkered(u32),
}

/// Used to load default meshes, meshes from files or user defined meshes.
struct MeshLoader {
    /// Scale the position of the vertices by this amount.
    scale: f32,
}
