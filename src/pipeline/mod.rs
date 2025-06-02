//! Contains everytihng that will be needed to render the scene.

use geometry::Geometry;
use glam::{DVec2, Vec3Swizzles, Vec4Swizzles};

use crate::{algorithm, graphics::screen::Screen, resources::texture::Texture, scene::Scene};

pub mod geometry;
mod transforms;

/// Contains cached values imprtant for rendering.
pub struct Pipeline {}

impl Pipeline {
    /// Constructs a pipeline.
    pub fn new() -> Self {
        Pipeline {}
    }
    /// Processes the data contained within the scene and prepares it for rendering.
    ///
    /// # Arguments
    ///
    /// * `scene` - The scene that will be processed. Every mesh withing will be rendererd.
    /// * `screen` - Where the scene will be rasterized.
    pub fn process_scene(&self, scene: &Scene, screen: &mut Screen) {
        let textures = scene.texture_catalog().textures();
        let camera = scene.camera();
        let projection = camera.projection();
        let camera_inv_transform = camera.transform().inverse();
        // Handle the scene differently ddepending on projection method.
        match projection {
            crate::scene::camera::Projection::Perspective {
                near_clip,
                far_clip,
                aspect_ratio,
                hfov,
            } => {
                // Get the camera perspective transform
                let perspective_transform =
                    transforms::perspective_transform(*near_clip, *far_clip, *aspect_ratio, *hfov);
                // Process all the meshes in order to rasterize them.
                for mesh in scene.meshes() {
                    let mut geometry = Geometry::from_mesh(mesh);
                    // Convert geometry to world coordinates.
                    geometry.lin_transform(mesh.transform());
                    // Do backface culling.
                    geometry.cull_backface(&camera.position());
                    // Convert geometry to view space.
                    geometry.lin_transform(&camera_inv_transform);
                    // Convert to clip space.
                    geometry.lin_transform(&perspective_transform);
                    // Clip trianlges to view frustum.
                    geometry.clip_geometry();
                    geometry.set_clip_w_inv();
                    // Convert to ndc space.
                    geometry.perspective_divide();
                    // Convert to screen space.
                    geometry.lin_transform(&transforms::ndc_to_screen_transform(
                        screen.width(),
                        screen.height(),
                    ));
                    // Rasterize to screen.
                    // First, get the geometry's texture.
                    let texture = if let Some(id) = geometry.texture_id() {
                        textures.get(&id)
                    } else {
                        None
                    };
                    self.rasterize(&geometry, screen, texture);
                }
            }
            crate::scene::camera::Projection::Orthographic { .. } => {
                todo!("Implement orthographic projection.");
            }
        }
    }
    /// Raterizes the geometry on the screen buffer.
    pub fn rasterize(&self, geometry: &Geometry, screen: &mut Screen, texture: Option<&Texture>) {
        // TODO: Use f32 instead of f64 for performance?
        let vertices = geometry.vertices();
        let uvs = geometry.uvs();
        let w_invs = geometry.clip_w_inv();
        let triangles = geometry.triangles();
        let (width, height) = (screen.width(), screen.height());
        let (frame, depth_buffer) = screen.buffers_mut();
        // Get number of channels the texture format requries (0 if no texture).
        let nb_channels = if let Some(t) = texture {
            t.nb_chanels() as usize
        } else {
            0
        };

        // Rasterize each triangle.
        for triangle_index_start in (0..triangles.len()).step_by(3) {
            // Triangle vertex indices.
            let (ai, bi, ci) = (
                triangles[triangle_index_start],
                triangles[triangle_index_start + 1],
                triangles[triangle_index_start + 2],
            );
            // Triangle vertex position in space.
            let (a, b, c) = (vertices[ai].xyz(), vertices[bi].xyz(), vertices[ci].xyz());
            // UV coordinates of each vertex.
            let (uv_a, uv_b, uv_c) = (uvs[ai], uvs[bi], uvs[ci]);
            // Inverted w (1/w) from the homogeneous coordinates in clip space.
            let (w_inv_a, w_inv_b, w_inv_c) = (w_invs[ai], w_invs[bi], w_invs[ci]);

            // The barycentric coordinate gradients.
            let (alpha_grad, beta_grad, gamma_grad) =
                algorithm::barycentric_gradients2(a.xy(), b.xy(), c.xy());
            // Initialize the important values' derivatives.
            let depth_dx = alpha_grad.x * a.z + beta_grad.x * b.z + gamma_grad.x * c.z;
            let w_inv_dx = alpha_grad.x * w_inv_a + beta_grad.x * w_inv_b + gamma_grad.x * w_inv_c; // Used for interpolation.
            let uv_over_w_dx = alpha_grad.x * uv_a * w_inv_a
                + beta_grad.x * uv_b * w_inv_b
                + gamma_grad.x * uv_c * w_inv_c;

            // Get bounding box of triangle.
            // max values are excluded, while min values are included.
            let (mut min_x, mut max_x, mut min_y, mut max_y) =
                Pipeline::triangle_aabs(a.xy(), b.xy(), c.xy());
            // Ensure they don't cross the screen's border.
            min_x = min_x.clamp(0, width);
            min_y = min_y.clamp(0, height);
            max_x = max_x.clamp(0, width);
            max_y = max_y.clamp(0, height);

            // Add 0.5 to both x and y in order to sample the pixel at its center.
            let min_pos = DVec2::new(min_x as f64 + 0.5, min_y as f64 + 0.5);

            // Get barycentric coordinates at min_pos.
            let (alpha_00, beta_00, gamma_00) = (
                alpha_grad.dot(min_pos - c.xy()),
                beta_grad.dot(min_pos - a.xy()),
                gamma_grad.dot(min_pos - b.xy()),
            );
            // Initialize coordinates for first row (redundant, but clearer)
            let (mut alpha_0y, mut beta_0y, mut gamma_0y) = (alpha_00, beta_00, gamma_00);

            // Rasterize over the bounding box.
            for y in min_y..max_y {
                let mut pixel_index = (min_x + y * width) as usize;
                // Initialize bayrcentric coordinates and other important values for the first x
                // value of the bounding square.
                let (mut alpha_xy, mut beta_xy, mut gamma_xy) = (alpha_0y, beta_0y, gamma_0y);
                let mut depth = alpha_xy * a.z + beta_xy * b.z + gamma_xy * c.z;
                let mut w_inv = alpha_xy * w_inv_a + beta_xy * w_inv_b + gamma_xy * w_inv_c;
                let mut uv_over_w = alpha_xy * uv_a * w_inv_a
                    + beta_xy * uv_b * w_inv_b
                    + gamma_xy * uv_c * w_inv_c; // Weird value, but useful given its linear
                                                 // properties in screen space.
                for _ in min_x..max_x {
                    // Check if pixel is inside the triangle.
                    if (alpha_xy >= 0.0) & (beta_xy >= 0.0) & (gamma_xy >= 0.0) {
                        // Make sure pixels closer to the screen have not been been drawn.
                        // Smaller depth means closer to screen.
                        if depth < depth_buffer[pixel_index] {
                            depth_buffer[pixel_index] = depth;

                            // Get the UV coordinates of the pixel.
                            let uv = uv_over_w / w_inv;

                            // Given the UV coordinates, get the texture color and draw it.
                            let pixel_channel_index = 4 * pixel_index;
                            match texture {
                                Some(texture) => {
                                    let color = texture.from_uv(uv[0], uv[1]);
                                    // SAFETY: frame is guaranteed to have at least 4 valid indices
                                    // after pixel_channel_index, and color has at most 4. Thus,
                                    // when copying, nothing will go out of bounds.
                                    unsafe {
                                        std::ptr::copy_nonoverlapping(
                                            color.as_ptr(),
                                            frame.as_mut_ptr().add(pixel_channel_index),
                                            3,
                                        );
                                    }
                                    // If texture didn't have an alpha channel, use max alpha.
                                    if nb_channels != 4 {
                                        frame[pixel_channel_index + 3] = 255;
                                    }
                                }
                                // Black if no texture.
                                None => {
                                    frame[pixel_channel_index..pixel_channel_index + 4]
                                        .copy_from_slice(&[0, 0, 0, 255]);
                                }
                            };
                        }
                    }
                    // Update barycentric coordinates for next horizontal pixel.
                    alpha_xy += alpha_grad.x;
                    beta_xy += beta_grad.x;
                    gamma_xy += gamma_grad.x;

                    // Update important values with their derivatives for the next horizontal pixel.
                    depth += depth_dx;
                    w_inv += w_inv_dx;
                    uv_over_w += uv_over_w_dx;

                    pixel_index += 1;
                }
                // Update barycentric coordinates for next row.
                alpha_0y += alpha_grad.y;
                beta_0y += beta_grad.y;
                gamma_0y += gamma_grad.y;
            }
        }
    }
    /// Given the vertices of a triangle, obtain its axis aligned bounding square.
    ///
    /// # Return
    ///
    /// The maximum and minimum values of x and y of the bounding square
    /// in the following format:
    /// (min_x, max_x, min_y, max_y)
    fn triangle_aabs(a: DVec2, b: DVec2, c: DVec2) -> (usize, usize, usize, usize) {
        let mut min_x = a.x.min(b.x).min(c.x).floor();
        let mut max_x = a.x.max(b.x).max(c.x).ceil();
        let mut min_y = a.y.min(b.y).min(c.y).floor();
        let mut max_y = a.y.max(b.y).max(c.y).ceil();
        min_x = if min_x < 0.0 { 0.0 } else { min_x };
        max_x = if max_x < 0.0 { 0.0 } else { max_x };
        min_y = if min_y < 0.0 { 0.0 } else { min_y };
        max_y = if max_y < 0.0 { 0.0 } else { max_y };
        (
            min_x as usize,
            max_x as usize,
            min_y as usize,
            max_y as usize,
        )
    }
}
