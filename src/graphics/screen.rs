//! Handles actions related to screen drawing.
use std::sync::Arc;

use pixels::{self, Pixels, SurfaceTexture};
use winit;

use crate::resources::texture::Texture;

/// Contains the necessary information to draw pixels on the screen.
pub struct Screen {
    /// Width of the buffer.
    width: u32,
    /// Height of the buffer.
    height: u32,
    /// Pixels instance used to draw on screen.
    pixels: Option<pixels::Pixels<'static>>,
    /// Buffer containing pixel depths.
    depth_buffer: Vec<f64>,
}

impl Screen {
    /// Creates new screen.
    ///
    /// Creates new screen, but does not instantiate pixels.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the screen buffer.
    /// * `height` - Height of the screen buffer.
    ///
    /// # Returns
    ///
    /// The instantiated Screen.
    pub fn new(width: u32, height: u32) -> Self {
        Screen {
            width,
            height,
            pixels: None,
            depth_buffer: vec![f64::MAX; (width * height) as usize],
        }
    }
    /// Initializes the pixels instance.
    ///
    /// Given a shared winit window, it creates the pixels instance.
    ///
    /// # Arguments
    ///
    /// * `window` - Shared window that hosts the application.
    ///
    /// # Returns
    ///
    /// An error if instantiation fails or nothing if everything goes well.
    pub fn initialize_pixels(
        &mut self,
        window: Arc<winit::window::Window>,
    ) -> Result<(), pixels::Error> {
        let pixels = {
            let surface_texture = SurfaceTexture::new(self.width, self.height, window);
            pixels::Pixels::new(self.width, self.height, surface_texture)?
        };
        self.pixels = Some(pixels);
        Ok(())
    }
    /// Clears the screen to black, and resets depth buffer.
    pub fn screen_clear(&mut self) {
        // Reset screen.
        for pixel in self.pixels_mut().unwrap().frame_mut().chunks_exact_mut(4) {
            pixel.copy_from_slice(&[42, 0, 23, 255]);
        }
        self.depth_buffer = vec![f64::MAX; (self.width * self.height) as usize];
    }
    /// Draws a texture on the screen. Where 0,0 on the screen is 0,0 uv, and width, height, is 1,1
    /// uv.
    pub fn draw_texture(&mut self, texture: &Texture) {
        let width = self.width as usize;
        let height = self.height as usize;
        let frame = self.pixels_mut().unwrap().frame_mut();
        let nb_channels = match texture.format() {
            crate::resources::texture::Format::RGBA32 => 4,
            crate::resources::texture::Format::RGB24 => 3,
        };
        for row in 0..height as usize {
            for col in 0..width as usize {
                let (u, v) = (col as f64 / width as f64, row as f64 / height as f64);
                let pixel = texture.from_uv(u, v);
                let index = (row * width as usize + col) * 4 as usize;
                frame[index..index + nb_channels].copy_from_slice(pixel);
                if nb_channels == 4 {
                    frame[index + 3] = 255;
                }
            }
        }
    }
}
// Getters and setters.
impl Screen {
    /// Mutable reference for the pixels instance.
    pub fn pixels_mut(&mut self) -> Option<&mut Pixels<'static>> {
        self.pixels.as_mut()
    }
    /// Mutable reference for the pixels depth buffer.
    pub fn depth_buffer_mut(&mut self) -> &mut Vec<f64> {
        &mut self.depth_buffer
    }
    /// Getter for screen width.
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Getter for screen height.
    pub fn height(&self) -> u32 {
        self.height
    }
}
