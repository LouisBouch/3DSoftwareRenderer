//! Contains everytihng that will be needed to render the scene.

use std::collections::HashMap;

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
                    // Convert geometry to view space.
                    geometry.lin_transform(camera.transform());
                    // Do backface culling.
                    geometry.cull_backface(&camera.position());
                    // Convert to clip space.
                    geometry.lin_transform(&perspective_transform);
                    geometry.set_clip_w_inv();
                    // Clip trianlges to view frustum.
                    geometry.clip_geometry();
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
                    //screen.draw() (do it somewhere else?)
                }
            }
            crate::scene::camera::Projection::Orthographic { .. } => {
                todo!("Implement orthographic projection.");
            }
        }
    }
    /// Raterizes the geometry on the screen buffer.
    pub fn rasterize(&self, geometry: &Geometry, screen: &mut Screen, texture: Option<&Texture>) {
        let vertices = geometry.vertices();
        let uvs = geometry.uvs();
        let w_invs = geometry.clip_w_inv();
        let triangles = geometry.triangles();
        let (width, height) = (screen.width(), screen.height());
        // Get number of channels the texture format requries (0 if no texture).
        let nb_channels = if let Some(t) = texture {
            t.nb_chanels() as usize
        } else {
            0
        };

        // Rasterize each triangle.
        for triangle_index_start in (0..triangles.len()).step_by(3) {
            let (a, b, c) = (
                vertices[triangles[triangle_index_start as usize] as usize].xyz(),
                vertices[triangles[triangle_index_start as usize + 1] as usize].xyz(),
                vertices[triangles[triangle_index_start as usize + 2] as usize].xyz(),
            );
            let (uv_a, uv_b, uv_c) = (
                uvs[triangles[triangle_index_start as usize] as usize],
                uvs[triangles[triangle_index_start as usize + 1] as usize],
                uvs[triangles[triangle_index_start as usize + 2] as usize],
            );
            let (w_inv_a, w_inv_b, w_inv_c) = (
                w_invs[triangles[triangle_index_start as usize] as usize],
                w_invs[triangles[triangle_index_start as usize + 1] as usize],
                w_invs[triangles[triangle_index_start as usize + 2] as usize],
            );

            // The barycentric coordinate gradients.
            let (grad_alpha, grad_beta, grad_gamma) =
                algorithm::barycentric_gradients2(a.xy(), b.xy(), c.xy());

            // Get bounding box of triangle.
            // max values are excluded, while min values are included.
            let (min_x, max_x, min_y, max_y) = Pipeline::triangle_aabs(a.xy(), b.xy(), c.xy());
            // Ensure they don't cross the screen's border.
            let min_x = if min_x > width { width } else { min_x };
            let max_x = if max_x > width { width } else { max_x };
            let min_y = if min_y > height { height } else { min_y };
            let max_y = if max_y > height { height } else { max_y };

            // Get barycentric coordinates at screen pos (min_x, min_y).
            let min_pos = DVec2::new(min_x as f64, min_y as f64);
            let (alpha_00, beta_00, gamma_00) = (
                grad_alpha.dot(min_pos - c.xy()),
                grad_beta.dot(min_pos - a.xy()),
                grad_gamma.dot(min_pos - b.xy()),
            );
            // Initialize coordinates for first row (redundant, but clearer)
            let (mut alpha_0y, mut beta_0y, mut gamma_0y) = (alpha_00, beta_00, gamma_00);
            // Rasterize over the bounding box.
            for y in min_y..max_y {
                // Initialize column bayrcentric coordinates.
                let (mut alpha_xy, mut beta_xy, mut gamma_xy) = (alpha_0y, beta_0y, gamma_0y);
                for x in min_x..max_x {
                    let pixel_index = x + y * width;
                    // Update coordinates to next column.
                    if x != min_x {
                        alpha_xy += grad_alpha.x;
                        beta_xy += grad_beta.x;
                        gamma_xy += grad_gamma.x;
                    }

                    // Check if pixel is outside the triangle.
                    if (alpha_xy < 0.0) && (beta_xy < 0.0) && (gamma_xy < 0.0) {
                        continue;
                    }
                    // Check if pixel closer to the screen has already been drawn.
                    // Smaller depth means closer to screen.
                    let depth = alpha_xy * a.z + beta_xy * b.z + gamma_xy * c.z;
                    let depth_buffer = screen.depth_buffer_mut();
                    if depth as f32 >= depth_buffer[pixel_index as usize] {
                        continue;
                    }
                    // Get the w inverse of the pixel (used for interpolation).
                    let w_inv = alpha_xy * w_inv_a + beta_xy * w_inv_b + gamma_xy * w_inv_c;

                    // Get the UV coordinates of the pixel.
                    let uv = (alpha_xy * uv_a * w_inv_a
                        + beta_xy * uv_b * w_inv_b
                        + gamma_xy * uv_c * w_inv_c)
                        / w_inv;
                    // Given the UV coordinates, get the texture color.
                    let frame = screen.pixels_mut().unwrap().frame_mut();
                    let pixel_value: [u8; 4] = match texture {
                        Some(texture) => {
                            let mut pixel: [u8; 4] = [0, 0, 0, 255];
                            let slice = texture.from_uv(uv[0], uv[1]);
                            pixel[..].copy_from_slice(&slice[0..nb_channels]);
                            pixel
                        }
                        // Black if no texture.
                        None => [0, 0, 0, 255],
                    };
                    // Now draw it.
                    frame[pixel_index as usize..pixel_index as usize + 4]
                        .copy_from_slice(&pixel_value);
                }
                // Update barycentric coordinates for next row.
                alpha_0y += grad_alpha.y;
                beta_0y += grad_beta.y;
                gamma_0y += grad_gamma.y;
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
    fn triangle_aabs(a: DVec2, b: DVec2, c: DVec2) -> (u32, u32, u32, u32) {
        let mut min_x = a.x.min(b.x).min(c.x).floor();
        let mut max_x = a.x.max(b.x).max(c.x).ceil();
        let mut min_y = a.y.min(b.y).min(c.y).floor();
        let mut max_y = a.y.max(b.y).max(c.y).ceil();
        min_x = if min_x < 0.0 { 0.0 } else { min_x };
        max_x = if max_x < 0.0 { 0.0 } else { max_x };
        min_y = if min_y < 0.0 { 0.0 } else { min_y };
        max_y = if max_y < 0.0 { 0.0 } else { max_y };
        (min_x as u32, max_x as u32, min_y as u32, max_y as u32)
    }
}
