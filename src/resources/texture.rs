//! Contains everything pertaining to texture storage.
use core::fmt;
use std::collections::HashMap;

/// Owns the textures as well as the necessary maps to efficiently access them.
pub struct TextureCatalog {
    /// Id to give to the next texture added. Start at 1, and use 0 for issues.
    next_id: u32,
    /// Map containing the id of each texture.
    textures: HashMap<u32, Texture>,
    /// Given a texture name (file name), obtain the id of the texture.
    texture_ids: HashMap<String, u32>,
}
impl TextureCatalog {
    /// Creates a default texture catalog with no textures.
    pub fn new() -> Self {
        TextureCatalog {
            next_id: 1,
            textures: HashMap::new(),
            texture_ids: HashMap::new(),
        }
    }
    /// Add a teture to the catalog.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the texture.
    /// * `texture` - The texture to add to the catalog.
    ///
    /// # Return
    ///
    /// The id of the texture that was added to the catalog if succesful.
    pub fn add_texture(&mut self, name: String, texture: Texture) -> Result<u32, TextureError> {
        if let Some(&id) = self.texture_ids.get(&name) {
            return Err(TextureError::TextureNameAlreadyExists { name: name, id: id });
        }
        self.texture_ids.insert(name, self.next_id);
        self.textures.insert(self.next_id, texture);
        self.next_id += 1;
        Ok(self.next_id - 1)
    }
    /// Get the texture from its id
    ///
    /// # Arguments
    ///
    /// * `id` - Id of the texture.
    ///
    /// # Return
    ///
    /// A reference to the texture associated with the `id`. None otherwise.
    pub fn texture_from_id(&self, id: u32) -> Option<&Texture> {
        self.textures.get(&id)
    }
    /// Get the id of a texture given its name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the texture.
    ///
    /// # Return
    ///
    /// The id associated with the `name`, if it exists. None otherwise.
    pub fn id_from_name(&self, name: &str) -> Option<u32> {
        self.texture_ids.get(name).copied()
    }
}
/// The texture defined as a 2D image of pixels.
pub struct Texture {
    /// The RGB/A pixel values for every pixels. Left to right, top to bottom.
    pub pixels: Vec<u8>,
    /// Number of pixels horizontally.
    pub width: u32,
    /// Number of pixels vertically.
    pub height: u32,
    /// Pixel format of the texture.
    pub format: Format,
}
impl Texture {
    /// Create a new invisible texture instance.
    ///
    /// # Arguments
    ///
    /// * `width` - The width (in pixels) of the texture.
    /// * `height` - The height (in pixels) of the texture.
    /// * `format` - The pixel format used to create the texture.
    ///
    /// # Return
    ///
    /// The new instance created through the function.
    pub fn new(width: u32, height: u32, format: Format) -> Self {
        match format {
            Format::RGBA32 => Texture {
                pixels: vec![0; 4 * width as usize * height as usize],
                width,
                height,
                format,
            },
            Format::RGB24 => Texture {
                pixels: vec![0; 3 * width as usize * height as usize],
                width,
                height,
                format,
            },
        }
    }
    /// Create a new user defined texture.
    ///
    /// # Arguments
    ///
    /// * `pixels` - The list of bytes defining the texture's pixels.
    /// * `width` - The width (in pixels) of the texture.
    /// * `height` - The height (in pixels) of the texture.
    /// * `format` - The pixel format used to create the texture.
    ///
    /// # Return
    ///
    /// The new instance created through the function.
    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: &Vec<u8>,
        format: Format,
    ) -> Result<Self, TextureError> {
        // Check the number of channels that the format enforces.
        let format_channels = match format {
            Format::RGBA32 => 4,
            Format::RGB24 => 3,
        };
        // Check if pixels has correct size given width, height
        // and the number of channels.
        if width * height * format_channels != pixels.len() as u32 {
            return Err(TextureError::MismatchedPixelDataSize {
                expected: width * height * format_channels,
                actual: pixels.len() as u32,
            });
        }
        Ok(Texture {
            pixels: pixels.clone(),
            width,
            height,
            format,
        })
    }
}
/// Format of the texture.
pub enum Format {
    /// 8 bits for red, green, blue and alpha channels, respectively.
    RGBA32,
    /// 8 bits for red, green, blue channels, respectively.
    RGB24,
}
/// List of error that can be thrown when using textures.
#[derive(Debug)]
pub enum TextureError {
    /// Used when creating a texture from a list of pixels and the width, height and format given
    /// are incompatible with the pixel data received.
    MismatchedPixelDataSize {
        /// Expected length of the pixels vector.
        expected: u32,
        /// Actual length of the pixels array.
        actual: u32,
    },
    /// Used when the user tries to add a texture that already exists in the catalog.
    TextureNameAlreadyExists {
        /// Name of the texture.
        name: String,
        /// Id of the existing texture.
        id: u32,
    },
}

impl fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureError::MismatchedPixelDataSize {
                expected,
                actual: got,
            } => {
                write!(f, "Pixel data incompatible with given width, height and format. Expected {}, got {}", expected, got)
            }
            TextureError::TextureNameAlreadyExists { name, id } => {
                write!(
                    f,
                    "The texture with name '{}' already exists with id '{}'.",
                    name, id
                )
            }
        }
    }
}
impl std::error::Error for TextureError {}
