//! Contains everything pertaining to texture storage.
use core::fmt;
use std::collections::HashMap;

/// Owns the textures as well as the necessary maps to efficiently access them.
pub struct TextureCatalog {
    /// Map containing the id of each texture.
    textures: HashMap<u32, Texture>,
    /// Given a texture name (file name), obtain the id of the texture.
    texture_ids: HashMap<String, u32>,
}
/// The texture defined as a 2D image of pixels.
pub struct Texture {
    pixels: Vec<u8>,
    /// Number of pixels horizontally.
    width: u32,
    /// Number of pixels vertically.
    height: u32,
    /// Pixel format of the texture.
    format: Format,
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
        actual: u32 },
}

impl fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureError::MismatchedPixelDataSize { expected, actual: got } => {
                write!(f, "Pixel data incompatible with given width, height and format. Expected {}, got {}", expected, got)
            }
        }
    }
}
impl std::error::Error for TextureError {}
