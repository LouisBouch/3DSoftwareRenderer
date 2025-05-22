//! Handles actions related to screen drawing.
use std::sync::Arc;

use pixels::{self, Pixels, SurfaceTexture};
use winit;

/// Contains the necessary information to draw pixels on the screen.
pub struct Screen {
    /// Width of the buffer.
    width: u32,
    /// Height of the buffer.
    height: u32,
    /// Pixels instance used to draw on screen.
    pixels: Option<pixels::Pixels<'static>>,
    /// Buffer containing pixel depths.
    depth_buffer: Vec<f32>,
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
            depth_buffer: vec![f32::MAX; (width * height) as usize],
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
    /// Mutable getter for the pixels instance.
    pub fn pixels_mut(&mut self) -> Option<&mut Pixels<'static>> {
        self.pixels.as_mut()
    }
}
