//! Handles actions realted to window creation.
use std::sync::Arc;

use winit::dpi;
use winit::error::OsError;
use winit::event_loop;
use winit::window;

/// Contains the necessary information to move and create a window.
pub struct Window {
    /// Width of the window.
    width: u32,
    /// Height of the window.
    height: u32,
    /// Shared instance of the winit window.
    winit_window: Option<Arc<window::Window>>,
}

impl Window {
    /// Creates new window.
    ///
    /// Creates new window, but does not instantiate the winit window.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the window.
    /// * `height` - Height of the window.
    ///
    /// # Returns
    ///
    /// The instantiated Window.
    pub fn new(width: u32, height: u32) -> Window {
        Window {
            width: width,
            height: height,
            winit_window: None,
        }
    }
    /// Initializes the winit window.
    ///
    /// Given an event loop, it creates the winit window instance.
    ///
    /// # Arguments
    ///
    /// * `event_loop` - Loop that runs the application.
    ///
    /// # Returns
    ///
    /// An error if instantiation fails or nothing if everything goes well.
    pub fn initialize_window(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
    ) -> Result<(), OsError> {
        let size = dpi::LogicalSize::new(self.width as f64, self.height as f64);
        let attributes = window::Window::default_attributes()
            .with_title("Sofware renderer")
            .with_inner_size(size)
            .with_min_inner_size(size);
        let winit_window = event_loop.create_window(attributes)?;
        self.winit_window = Some(Arc::new(winit_window));
        Ok(())
    }
    /// Getter for the width.
    pub fn width(&self) -> u32{
        self.width
    }
    /// getter for the height.
    pub fn height(&self) -> u32{
        self.height
    }
    /// Mutable getter for the winit window.
    pub fn winit_window_mut(&self) -> Option<&Arc<window::Window>> {
        self.winit_window.as_ref()
    }
}
