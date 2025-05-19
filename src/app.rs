//! Exposes the API that will be used to create an interactable window that can be drawn on.

use crate::action::Action;
use crate::{
    graphics::{self, screen::Screen, window::Window},
    inputs,
    scene::{self, Scene},
};
use glam::DVec3;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState};
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
    fn handle_actions(&mut self) {
        let actions = self.input_state.collect_actions();
        for action in actions.iter() {
            match action {
                Action::MoveForwards => {
                    self.scene.camera.add_position(&DVec3::Y);
                }
                Action::MoveBackwards => {
                    self.scene.camera.add_position(&-DVec3::Y);
                }
                Action::MoveLeft => {
                    self.scene.camera.add_position(&-DVec3::X);
                }
                Action::MoveRight => {
                    self.scene.camera.add_position(&DVec3::X);
                }
                Action::MoveUp => {
                    println!("Up");
                }
                Action::MoveDown => {
                    println!("Down");
                }
                Action::RotateCamera { pitch, yaw } => {
                    println!("Rotate, pitch: {}, yaw: {}", pitch, yaw);
                }
            }
        }
    }
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
        // TODO: Use something similar to capture the mouse.
        // self.window
        //     .winit_window
        //     .as_mut()
        //     .unwrap()
        //     .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        //     .unwrap();
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed, stopping.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // println!("Redrawing requested.");
                let pixels = self.screen.pixels.as_mut().unwrap();
                let pixel_index: u32;
                let frame = pixels.frame_mut();
                let camera_pos = self.scene.camera.get_position();
                pixel_index = (camera_pos.x*4.0 - self.window.width as f64 * 4.0 * camera_pos.y) as u32;
                frame[pixel_index as usize] = 255;
                frame[pixel_index as usize + 1] = 0;
                frame[pixel_index as usize + 2] = 255;
                frame[pixel_index as usize + 3] = 255;
                // for i in 0..20000 {
                //     pixel_index = i * 4;
                //     frame[pixel_index as usize] = 255;
                //     frame[pixel_index as usize + 1] = 255;
                //     frame[pixel_index as usize + 2] = 255;
                //     frame[pixel_index as usize + 3] = 255;
                // }
                pixels.render().unwrap();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let key_state = event.state;
                let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key else {
                    return;
                };
                // Give the input state of the key to the input handler.
                match key_state {
                    ElementState::Pressed => self.input_state.press_key(key_code),
                    ElementState::Released => self.input_state.release_key(key_code),
                }
            }
            _ => {}
        }
    }
    fn device_event(
        &mut self,
        _event_loop: &event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                println!("Delta mouse: {:?}", delta);
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        // Handle actions.
        self.handle_actions();
        // Renders the screen into the pixel buffer.
        // Redraws the screen.
        self.window
            .winit_window
            .as_ref()
            .expect("Window should be initialized")
            .request_redraw();
    }
}
