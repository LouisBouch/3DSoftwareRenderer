//! Contains logic pertaining to the creation of linear transformation matrix.

use glam::{DMat4, DVec4};

/// Given the [`height`] and [`width`] of the screen, create a transform that convert from ndc to
/// screen coordinates.
pub fn ndc_to_screen_transform(width: u32, height: u32) -> DMat4 {
    let half_width: f64 = width as f64 / 2.0;
    let half_height: f64 = height as f64 / 2.0;
    DMat4::from_cols(
        DVec4::new(half_width, 0.0, 0.0, 0.0),
        DVec4::new(0.0, -half_height, 0.0, 0.0),
        DVec4::new(0.0, 0.0, 1.0, 0.0),
        DVec4::new(half_width, half_height, 0.0, 1.0),
    )
}
/// Obtain the view perspective matrix given a camera's view frustum.
///
/// # Arguments
///
/// * `near_clip` - The near clipping plane of the view frustum.
/// * `far_clip` - The far clipping plane of the view frustum.
/// * `aspect_ratio` - The aspect ratio of the view frustum (width/height).
/// * `hfov` - Horizontal fov of the view frustum (In degrees).
pub fn perspective_transform(near_clip: f32, far_clip: f32, aspect_ratio: f32, hfov: f32) -> DMat4 {
    // Get the left and right position of the view frustum's near clipping plane.
    // (Our perspective makes it such that -l=r)
    let r = ((hfov / 2.0).to_radians().tan() * near_clip) as f64;
    let l = -r;
    // Same for top and bottom (t, b).
    let t = r / aspect_ratio as f64;
    let b = -t;
    // Convert near and far clipping planes to f64.
    let n = near_clip as f64;
    let f = far_clip as f64;

    DMat4::from_cols(
        DVec4::new(2.0 * n / (r - l), 0.0, 0.0, 0.0),
        DVec4::new(0.0, 2.0 * n / (t - b), 0.0, 0.0),
        DVec4::new(
            (r + l) / (r - l),
            (t + b) / (t - b),
            -(f + n) / (f - n),
            -1.0,
        ),
        DVec4::new(0.0, 0.0, -(2.0 * f * n) / (f - n), 0.0),
    )
}
