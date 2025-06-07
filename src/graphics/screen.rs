//! Handles actions related to screen drawing.
use std::sync::Arc;

use pixels::{self, Pixels, SurfaceTexture};
use winit;

use crate::resources::texture::Texture;

/// Contains the necessary information to draw pixels on the screen.
pub struct Screen {
    /// Width of the buffer.
    width: usize,
    /// Height of the buffer.
    height: usize,
    /// Pixels instance used to draw on screen.
    pixels: Option<pixels::Pixels<'static>>,
    /// Background color.
    bg_color: [u8; 4],
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
    pub fn new(width: usize, height: usize) -> Self {
        Screen {
            width,
            height,
            pixels: None,
            bg_color: [42, 0, 23, 255],
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
            let surface_texture =
                SurfaceTexture::new(self.width as u32, self.height as u32, window);
            pixels::Pixels::new(self.width as u32, self.height as u32, surface_texture)?
        };
        self.pixels = Some(pixels);
        Ok(())
    }
    /// Clears the screen to black, and resets depth buffer.
    pub fn screen_clear(&mut self) {
        // Reset screen (Not needed when multithreading, as the threads create their own buffer
        // with the default color).
        // let color = self.bg_color;
        // Safe pixel reset.
        // for pixel in self.pixels_mut().unwrap().frame_mut().chunks_exact_mut(4) {
        //     pixel.copy_from_slice(&color);
        // }
        // Unsafe option. Doesn't seem faster than the safe one for some reason.
        // let coloru32 = u32::from_ne_bytes(color);
        // let frame = self.pixels.as_mut().unwrap().frame_mut();
        // let nb_pixels = frame.len() / 4;
        // unsafe {
        //     let frame_ptr = frame.as_mut_ptr() as *mut u32;
        //     for i in 0..nb_pixels {
        //         std::ptr::write_unaligned(frame_ptr.add(i), coloru32);
        //     }
        // }
    }
    /// Draws a texture on the screen. Where 0,0 on the screen is 0,0 uv, and width, height, is 1,1
    /// uv.
    pub fn draw_texture(&mut self, texture: &Texture) {
        let width = self.width;
        let height = self.height;
        let frame = self.pixels_mut().unwrap().frame_mut();
        let nb_channels = match texture.format() {
            crate::resources::texture::Format::RGBA32 => 4,
            crate::resources::texture::Format::RGB24 => 3,
        };
        for row in 0..height {
            for col in 0..width {
                let (u, v) = (col as f64 / width as f64, row as f64 / height as f64);
                let pixel = texture.from_uv(u, v);
                let index = (row * width + col) * 4 as usize;
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
    /// Getter for screen width.
    pub fn width(&self) -> usize {
        self.width
    }
    /// Getter for screen height.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Reference to the background color.
    pub fn bg_color(&self) -> &[u8] {
        &self.bg_color
    }
}
