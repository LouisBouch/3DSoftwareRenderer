//! Exposes the API that will be used to create an interactable window that can be drawn on.

use crate::{
    graphics::{self, screen::Screen, window::Window},
    inputs,
    scene::{self, Scene},
};
use crate::action::Action;
use winit::application::ApplicationHandler;
use winit::{event::WindowEvent, event_loop};

/// Contains the window, screen that is within the window and the input manager.
pub struct App {
    /// Contains the winit window and its dimension.
    window: graphics::window::Window,
    /// Contains the screen instance which will be used to draw on the window.
    screen: graphics::screen::Screen,
    /// Allows us to handle the user inputs.
    input_state: inputs::InputHandler,
    /// Contains everything needed to render the environment.
    scene: scene::Scene,
}
impl App {
    /// Creates an app.
    ///
    /// Prepares the necessary fields before running the event loop.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the window.
    /// * `height` - Height of the window.
    ///
    /// # Returns
    ///
    /// The instantiated App.
    pub fn new(width: u32, height: u32) -> Self {
        let window = Window::new(width, height);
        let input_state = inputs::InputHandler::new();
        let screen = Screen::new(width, height);
        let scene = {
            let camera = scene::camera::Camera::default();
            Scene { camera }
        };
        App {
            window,
            screen,
            input_state,
            scene,
        }
    }
    /// Acts on actions.
    ///
    /// Given a list of actions from the InputHandler, execute the required code for each.
    /// These actions will include mouse movements too, whose magnitude will need to be queried.
    ///
    /// # Arguments
    ///
    /// * `action` - List of actions to act upon.
    fn handle_actions(&mut self, actions: Vec<Action>) {}
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        // Initialize the winit window inside the app's window.
        if let Err(e) = self.window.initialize_window(event_loop) {
            eprintln!("Failed to initialize window: {e}");
            std::process::exit(1);
        }

        // Initialize the pixels instance inside the screen.
        let winit_window_shared = self
            .window
            .winit_window
            .clone()
            .expect("The window should be instantiated");
        if let Err(e) = self.screen.initialize_pixels(winit_window_shared) {
            eprintln!("Failed to initialize screen: {e}");
            std::process::exit(1);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed, stopping.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                println!("Redrawing requested.");
                let pixels = self.screen.pixels.as_mut().unwrap();
                let mut pixel_index: u32;
                let frame = pixels.frame_mut();
                for i in 0..20000 {
                    pixel_index = i * 4;
                    frame[pixel_index as usize] = 255;
                    frame[pixel_index as usize + 1] = 255;
                    frame[pixel_index as usize + 2] = 255;
                    frame[pixel_index as usize + 3] = 255;
                }
                pixels.render().unwrap();
                // Redraws the screen.
                self.window
                    .winit_window
                    .as_ref()
                    .expect("Window should be initialized")
                    .request_redraw();
            }
            _ => {}
        }
    }
}
