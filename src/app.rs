//! Exposes the API that will be used to create an interactable window that can be drawn on.

use crate::action::Action;
use crate::pipeline::Pipeline;
use crate::{
    graphics::{self, screen::Screen, window::Window},
    inputs,
    scene::{self, Scene},
};
use glam::{DVec2, DVec3};
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
    /// Number of frames per second.
    fps: u32,
    /// The pipeline that is used to transform the data into a rasterized image.
    pipeline: Pipeline,
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
    /// * `scene` - The scene that will be rendered.
    ///
    /// # Returns
    ///
    /// The instantiated App.
    pub fn new(width: u32, height: u32, scene: Scene) -> Self {
        let window = Window::new(width, height);
        let input_state = inputs::InputHandler::new();
        let screen = Screen::new(width, height);
        let fps = 100;
        let pipeline = Pipeline::new();
        App {
            window,
            screen,
            input_state,
            scene,
            fps,
            pipeline,
        }
    }
    /// Creates an app.
    ///
    /// Prepares the necessary fields before running the event loop and uses a default scene.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the window.
    /// * `height` - Height of the window.
    ///
    /// # Returns
    ///
    /// The instantiated App.
    pub fn with_default_scene(width: u32, height: u32) -> Self {
        let scene = Scene::new();
        Self::new(width, height, scene)
    }
    /// Acts on actions.
    ///
    /// Given a list of actions from the InputHandler, execute the required code for each.
    /// These actions will include mouse movements too, whose magnitude will need to be queried.
    fn handle_actions(&mut self) {
        let actions = self.input_state.collect_actions();
        let camera = self.scene.camera_mut();
        for action in actions.iter() {
            match action {
                Action::MoveForwards => {
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Forwards);
                }
                Action::MoveBackwards => {
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Backwards);
                }
                Action::MoveLeft => {
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Left);
                }
                Action::MoveRight => {
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Right);
                }
                Action::MoveUp => {
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Up);
                }
                Action::MoveDown => {
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Down);
                }
                Action::RotateCamera { pitch, yaw, roll } => {
                    camera.yaw(*yaw);
                    camera.pitch(*pitch);
                    camera.roll(*roll);
                    // // Rotate around world's y axis. Simulates FPS camera.
                    // let fixed_yaw_qat = DQuat::from_axis_angle(DVec3::Y, *yaw);
                    // camera.rotate(&fixed_yaw_qat);
                    //
                    // // Rotate around world's x axis. Simulates FPS camera.
                    // let fixed_pitch_qat = DQuat::from_axis_angle(DVec3::X, *pitch);
                    // camera.rotate(&fixed_pitch_qat);
                }
            }
        }
    }
    /// Sets the frames per second of the software renderer.
    pub fn set_fps(&mut self, fps: u32) {
        self.fps = fps;
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
            .winit_window_mut()
            .expect("The window should be instantiated")
            .clone();
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
                let pixels = self.screen.pixels_mut().unwrap();
                let pixel_index: u32;
                let frame = pixels.frame_mut();
                let camera_pos = self.scene.camera_mut().position();
                pixel_index =
                    (camera_pos.x * 4.0 - self.window.width() as f64 * 4.0 * camera_pos.y) as u32;
                frame[pixel_index as usize] = 255;
                frame[pixel_index as usize + 1] = 0;
                frame[pixel_index as usize + 2] = 255;
                frame[pixel_index as usize + 3] = 255;
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
                self.input_state
                    .mouse_move_raw(&DVec2::new(delta.0, delta.1));
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &event_loop::ActiveEventLoop) {
        // Handle actions.
        self.handle_actions();
        // Renders the screen into the pixel buffer.
        self.pipeline.process_scene(&self.scene, &mut self.screen);
        // Redraws the screen.
        self.window
            .winit_window_mut()
            .expect("Window should be initialized")
            .request_redraw();
    }
}
