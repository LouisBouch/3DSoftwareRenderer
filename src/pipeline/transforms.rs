//! Contains logic pertaining to the creation of linear transformation matrix.

use glam::{DMat4, DVec4};

/// Given the [`height`] and [`width`] of the screen, create a transform that convert from ndc to
/// screen coordinates.
pub fn ndc_to_screen_transform(width: u32, height: u32) -> DMat4 {
    let half_width: f64 = width as f64 / 2.0;
    let half_height: f64 = height as f64 / 2.0;
    DMat4 {
        x_axis: DVec4::new(half_width, 0.0, 0.0, half_width),
        y_axis: DVec4::new(0.0, half_height, 0.0, half_height),
        z_axis: DVec4::new(0.0, 0.0, 1.0, 0.0),
        w_axis: DVec4::new(0.0, 0.0, 0.0, 1.0),
    }
}
