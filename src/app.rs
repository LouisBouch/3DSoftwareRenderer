//! Exposes the API that will be used to create an interactable window that can be drawn on.

use std::time::{Duration, Instant};
use std::u32;

use crate::action::Action;
use crate::pipeline::Pipeline;
use crate::{
    graphics::{self, screen::Screen, window::Window},
    inputs,
    scene::{self, Scene},
};
use glam::DVec2;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState};
use winit::event_loop::ControlFlow;
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
    /// The time at which the next frame needs to render in order to hit the wanted fps.
    next_frame_time: Instant,
    /// Whether the mouse is captured within the app or not.
    mouse_captured: bool,
    /// Number of iterations before stopping the app (Mostly used for debuggin).
    max_it: u64,
    /// The current iteration number.
    cur_it: u64,
    /// Stores the last time the fps counter counted frames.
    last_fps_count_time: Instant,
    /// The number of frames that were rendered since the last fps count.
    frame_count: u32,
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
    pub fn new(width: usize, height: usize, scene: Scene) -> Self {
        let window = Window::new(width, height);
        let input_state = inputs::InputHandler::new();
        let screen = Screen::new(width, height);
        let fps = 144;
        let pipeline = Pipeline::new(64, width, height);
        let last_frame_time = Instant::now();
        let last_fps_count = Instant::now();

        let frame_count = 0;
        App {
            window,
            screen,
            input_state,
            scene,
            fps,
            pipeline,
            next_frame_time: last_frame_time,
            mouse_captured: false,
            max_it: u64::MAX,
            cur_it: 0,
            last_fps_count_time: last_fps_count,
            frame_count,
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
    pub fn with_default_scene(width: usize, height: usize) -> Self {
        let scene = Scene::new();
        Self::new(width, height, scene)
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
                    let camera = self.scene.camera_mut();
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Forwards);
                }
                Action::MoveBackwards => {
                    let camera = self.scene.camera_mut();
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Backwards);
                }
                Action::MoveLeft => {
                    let camera = self.scene.camera_mut();
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Left);
                }
                Action::MoveRight => {
                    let camera = self.scene.camera_mut();
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Right);
                }
                Action::MoveUp => {
                    let camera = self.scene.camera_mut();
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Up);
                }
                Action::MoveDown => {
                    let camera = self.scene.camera_mut();
                    camera.move_cam(1.0 / (self.fps as f64), scene::camera::Direction::Down);
                }
                Action::RotateCamera { pitch, yaw, roll } => {
                    if self.mouse_captured {
                        let camera = self.scene.camera_mut();
                        camera.yaw_pitch_roll(*yaw, *pitch, *roll);
                    }
                }
                Action::ToggleMouseCapture => {
                    self.capture_mouse(!self.mouse_captured);
                }
                Action::AddCameraVelocity(velocity) => {
                    self.scene.camera_mut().add_velocity(*velocity);
                }
            }
        }
    }
    /// Captures or release the mouse from the app.
    pub fn capture_mouse(&mut self, capture: bool) {
        let winit_window = self.window.winit_window_mut();
        // Capture mouse and make it invisible.
        match winit_window {
            Some(w) => {
                if capture {
                    w.set_cursor_visible(false);
                    w.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                        .unwrap_or_else(|e| {
                            eprintln!("Could not capture mouse: {e}");
                            w.set_cursor_visible(true);
                        });
                } else {
                    w.set_cursor_visible(true);
                    w.set_cursor_grab(winit::window::CursorGrabMode::None)
                        .unwrap_or_else(|e| {
                            eprintln!("Could not release mouse: {e}");
                        });
                }
            }
            _ => {}
        }
        self.mouse_captured = capture;
    }
    /// Renders the screen and handles movement of the camera.
    fn next_frame(&mut self) {
        // Check fps count (at most once every second).
        let now = Instant::now();
        if self.last_fps_count_time + Duration::from_secs(1) <= now {
            // Print fps count to window title.
            self.window.add_window_name_suffix(&format!(
                " (FPS: {:.2})",
                self.frame_count as f64 / (now - self.last_fps_count_time).as_secs_f64()
            ));
            self.last_fps_count_time = now;
            self.frame_count = 0;
        }
        // Handle actions.
        self.handle_actions();
        // Renders the screen into the pixel buffer.
        self.pipeline.process_scene(&self.scene, &mut self.screen);
        // self.screen.draw_texture(self.scene.texture_catalog().textures().get(&1).unwrap());

        self.cur_it += 1;
        self.frame_count += 1;
    }
}
// Getters/Setters
impl App {
    /// Sets the frames per second of the software renderer.
    pub fn set_fps(&mut self, fps: u32) {
        self.fps = fps;
    }
    /// Getter for maximum number of iterations.
    pub fn max_it(&self) -> u64 {
        self.max_it
    }
    /// Setter for maximum number of iterations.
    pub fn set_max_it(&mut self, max_it: u64) {
        self.max_it = max_it;
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
                // Get pixels.
                let pixels = self.screen.pixels_mut().unwrap();

                // Render them.
                // TODO: Verify is render is completed before frame is mutated in the renderer.
                // If not fully complete, this woudl explain the artifacts present when moving in
                // the scene. Might just be a V-Sync problem.
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
            WindowEvent::Focused(focused) => {
                self.capture_mouse(focused);
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
            DeviceEvent::MouseWheel { delta } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_, row) => {
                    if row < 0.0 {
                        self.input_state.add_nb_scrolls(1);
                    } else {
                        self.input_state.add_nb_scrolls(-1);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        // Check if we are at the last iteration.
        if self.cur_it >= self.max_it {
            event_loop.exit();
            println!("Max iteration achieved, closing app.");
        }

        // If enough time has passed, render a frame.
        if Instant::now() >= self.next_frame_time {
            // This value is set now instead of after next_frame to make sure
            // slow frames don't get slowed down a further 1/fps seconds.
            self.next_frame_time = Instant::now() + Duration::from_secs_f64(1.0 / self.fps as f64);

            // Reset screen.
            self.pipeline.clear();

            // Compute frame.
            self.next_frame();

            // Ask for next frame.
            self.window
                .winit_window_mut()
                .expect("Window should be initialized")
                .request_redraw();
        }

        // Wait until next frame before rendering again.
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
    }
}
