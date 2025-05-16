// use glam::vec2;
// use ndarray;
// let a: ndarray::Array2<i32> = ndarray::Array2::from_shape_vec((2, 3), b).unwrap();
// let b = vec![1, 2, 3, 4, 5, 6];
// let v = vec2(1.0, 1.0);
// println!("{}, {}", v.x, a[(1, 2)]);
use soft_rend::app::App;
use winit::event_loop::EventLoop;
fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(800, 600);
    event_loop.run_app(&mut app)?;
    Ok(())
}
