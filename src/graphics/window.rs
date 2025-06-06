//! Handles actions realted to window creation.
use std::sync::Arc;

use winit::dpi;
use winit::error::OsError;
use winit::event_loop;
use winit::window;

/// Contains the necessary information to move and create a window.
pub struct Window {
    /// Width of the window.
    width: usize,
    /// Height of the window.
    height: usize,
    /// Shared instance of the winit window.
    winit_window: Option<Arc<window::Window>>,
    /// The name of the window.
    window_name: String,
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
    pub fn new(width: usize, height: usize) -> Window {
        Window {
            width: width,
            height: height,
            winit_window: None,
            window_name: String::from("Software Renderer"),
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
            .with_title(&self.window_name)
            .with_inner_size(size)
            .with_min_inner_size(size);
        let winit_window = event_loop.create_window(attributes)?;
        self.winit_window = Some(Arc::new(winit_window));
        Ok(())
    }
    /// Reference for the width.
    pub fn width(&self) -> usize{
        self.width
    }
    /// Reference for the height.
    pub fn height(&self) -> usize{
        self.height
    }
    /// Mutable getter for the winit window.
    pub fn winit_window_mut(&self) -> Option<&Arc<window::Window>> {
        self.winit_window.as_ref()
    }
    /// Reference to the name of the window.
    pub fn window_name(&self) -> &str {
        &self.window_name
    }
    /// Add a suffix to the window name.
    pub fn add_window_name_suffix(&mut self, suffix: &str) {
        if let Some(winit_w) = self.winit_window.as_mut() {
            winit_w.set_title(&(self.window_name.clone() + suffix));
        }
    }
}
