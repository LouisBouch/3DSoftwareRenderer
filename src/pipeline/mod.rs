//! Contains everytihng that will be needed to render the scene.

use geometry::Geometry;
use glam::{DVec2, Vec3Swizzles, Vec4Swizzles};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

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
            // Triangle's vertex positions in space.
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
            // Both max and min values are included
            let (min_xf64, max_xf64, min_yf64, max_yf64) =
                algorithm::triangle_aabs(a.xy(), b.xy(), c.xy());
            // Ensure they don't cross the screen's border, and convert them to
            // integer screen coordinates.
            let min_x = min_xf64.max(0.0) as usize;
            let min_y = min_yf64.max(0.0) as usize;
            let max_x = (max_xf64 as usize).min(width - 1);
            let max_y = (max_yf64 as usize).min(height - 1);

            // Add 0.5 to both x and y in order to sample the pixel at its center.
            let min_posf64 = DVec2::new(min_x as f64 + 0.5, min_y as f64 + 0.5);

            // Get barycentric coordinates at min_pos.
            let (alpha_00, beta_00, gamma_00) = (
                alpha_grad.dot(min_posf64 - c.xy()),
                beta_grad.dot(min_posf64 - a.xy()),
                gamma_grad.dot(min_posf64 - b.xy()),
            );
            // Initialize coordinates for first row (redundant, but clearer)
            let (mut alpha_0y, mut beta_0y, mut gamma_0y) = (alpha_00, beta_00, gamma_00);

            // Rasterize over the bounding box.
            for y in min_y..=max_y {
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
                for _ in min_x..=max_x {
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
    /// Raterizes the geometry on the screen buffer while making use of multithreading.
    ///
    /// Uses a tilling approach, where the screen is divided into different
    /// tiles of size `tile_size`² and each one is rasterized by a different using rayon.
    pub fn rasterize_threaded(
        &self,
        geometry: &Geometry,
        screen: &mut Screen,
        texture: Option<&Texture>,
        tile_size: usize,
    ) {
        // Get useful values for rasterizing.
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

        // Figure out how many tiles are required given its size.
        let (nb_tile_x, nb_tile_y) = (
            (width + tile_size - 1) / tile_size,
            (height + tile_size - 1) / tile_size,
        );

        // Create tile buffers for the frame and depth.
        let mut depth_buffers: Vec<Vec<f64>> =
            vec![Vec::with_capacity(tile_size * tile_size); nb_tile_x * nb_tile_y];
        let mut frame_buffers: Vec<Vec<u8>> =
            vec![Vec::with_capacity(tile_size * tile_size * 4); nb_tile_x * nb_tile_y];

        // Bin the triangles into the different tiles.
        let mut binned_triangles: Vec<Vec<BinnedTriangle>> =
            vec![Vec::new(); nb_tile_x * nb_tile_y];

        // Start binning the triangles.
        for triangle_index_start in (0..triangles.len()).step_by(3) {
            // Triangle vertex indices.
            let (ai, bi, ci) = (
                triangles[triangle_index_start],
                triangles[triangle_index_start + 1],
                triangles[triangle_index_start + 2],
            );
            // Triangle vertex position in space.
            let (a, b, c) = (vertices[ai].xyz(), vertices[bi].xyz(), vertices[ci].xyz());

            // Get bounding box of triangle.
            // Both max and min values are included
            let (min_xf64, max_xf64, min_yf64, max_yf64) =
                algorithm::triangle_aabs(a.xy(), b.xy(), c.xy());
            // Ensure they don't cross the screen's border, and convert them to
            // integer screen coordinates.
            let min_x = min_xf64.max(0.0) as usize;
            let min_y = min_yf64.max(0.0) as usize;
            let max_x = (max_xf64 as usize).min(width - 1);
            let max_y = (max_yf64 as usize).min(height - 1);

            // Given the triangle's corner positions, find which tiles it intersects.
            let (first_tile_x, last_tile_x, first_tile_y, last_tile_y) = (
                min_x / tile_size,
                max_x / tile_size,
                min_y / tile_size,
                max_y / tile_size,
            );
            // Add the triangle to the bin of each tile.
            for tile_x in first_tile_x..=last_tile_x {
                for tile_y in first_tile_y..=last_tile_y {
                    let mut binned_triangle = BinnedTriangle::new();
                    // Get the relative position of the aabs within the tile.
                    binned_triangle.min_x = min_x - (tile_x * tile_size).max(min_x);
                    binned_triangle.min_y = min_y - (tile_y * tile_size).max(min_y);
                    binned_triangle.max_x = (max_x - tile_x * tile_size).min(tile_size);
                    binned_triangle.max_y = (max_y - tile_y * tile_size).min(tile_size);
                    binned_triangle.triangle_start = triangle_index_start;
                    // Push it in the corresponding bin.
                    binned_triangles[tile_x + tile_y * nb_tile_y].push(binned_triangle);
                }
            }
        }
        // Rasterize in parallel on each tile.
        frame_buffers
            .par_iter_mut()
            .zip(depth_buffers.par_iter_mut())
            .enumerate()
            .for_each(|(tile_nb, (tile_frame_buffer, tile_depth_buf))| {
                // Rasterize each triangle inside the tile.
                let binned_triangles_tile: &[BinnedTriangle] = &binned_triangles[tile_nb];
                for binned_triangle in binned_triangles_tile.iter() {
                    // Get the first vertex position of the triangle.
                    let triangle_index_start = binned_triangle.triangle_start;

                    // Triangle vertex indices.
                    let (ai, bi, ci) = (
                        triangles[triangle_index_start],
                        triangles[triangle_index_start + 1],
                        triangles[triangle_index_start + 2],
                    );
                    // Triangle's vertex positions in space.
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
                    let w_inv_dx =
                        alpha_grad.x * w_inv_a + beta_grad.x * w_inv_b + gamma_grad.x * w_inv_c; // Used for interpolation.
                    let uv_over_w_dx = alpha_grad.x * uv_a * w_inv_a
                        + beta_grad.x * uv_b * w_inv_b
                        + gamma_grad.x * uv_c * w_inv_c;

                    // Get bounding box of triangle within the tile.
                    let min_x = binned_triangle.min_x;
                    let min_y = binned_triangle.min_y;
                    let max_x = binned_triangle.max_x;
                    let max_y = binned_triangle.max_y;

                    // Add 0.5 to both x and y in order to sample the pixel at its center.
                    let min_posf64 = DVec2::new(min_x as f64 + 0.5, min_y as f64 + 0.5);

                    // Get barycentric coordinates at min_pos.
                    let (alpha_00, beta_00, gamma_00) = (
                        alpha_grad.dot(min_posf64 - c.xy()),
                        beta_grad.dot(min_posf64 - a.xy()),
                        gamma_grad.dot(min_posf64 - b.xy()),
                    );
                    // Initialize coordinates for first row (redundant, but clearer)
                    let (mut alpha_0y, mut beta_0y, mut gamma_0y) = (alpha_00, beta_00, gamma_00);

                    // Rasterize over the bounding box.
                    for y in min_y..=max_y {
                        let mut pixel_index = min_x + y * tile_size;
                        // Initialize bayrcentric coordinates and other important values for the first x
                        // value of the bounding square.
                        let (mut alpha_xy, mut beta_xy, mut gamma_xy) =
                            (alpha_0y, beta_0y, gamma_0y);
                        let mut depth = alpha_xy * a.z + beta_xy * b.z + gamma_xy * c.z;
                        let mut w_inv = alpha_xy * w_inv_a + beta_xy * w_inv_b + gamma_xy * w_inv_c;
                        let mut uv_over_w = alpha_xy * uv_a * w_inv_a
                            + beta_xy * uv_b * w_inv_b
                            + gamma_xy * uv_c * w_inv_c; // Weird value, but useful given its linear
                                                         // properties in screen space.
                        for _ in min_x..=max_x {
                            // Check if pixel is inside the triangle.
                            if (alpha_xy >= 0.0) & (beta_xy >= 0.0) & (gamma_xy >= 0.0) {
                                // Make sure pixels closer to the screen have not been been drawn.
                                // Smaller depth means closer to screen.
                                if depth < tile_depth_buf[pixel_index] {
                                    tile_depth_buf[pixel_index] = depth;

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
                                                    tile_frame_buffer
                                                        .as_mut_ptr()
                                                        .add(pixel_channel_index),
                                                    3,
                                                );
                                            }
                                            // If texture didn't have an alpha channel, use max alpha.
                                            if nb_channels != 4 {
                                                tile_frame_buffer[pixel_channel_index + 3] = 255;
                                            }
                                        }
                                        // Black if no texture.
                                        None => {
                                            tile_frame_buffer
                                                [pixel_channel_index..pixel_channel_index + 4]
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
            });
        // Write back to the main frame/depth buffers.
        // Use concurrency?
        for tile_y in 0..nb_tile_y {
            let mut tile_nb = tile_y * nb_tile_x;
            for tile_x in 0..nb_tile_x {
                // Get references to the buffers.
                let tile_frame_buffer: &[u8] = &frame_buffers[tile_nb];
                let tile_depth_buffer: &[f64] = &depth_buffers[tile_nb];

                // Get pixel offset.
                let first_pixel_index = tile_x * tile_size + tile_y * tile_size * width;

                // For each row in the tile, copy it over to the main frame buffer.
                for tile_row in 0..tile_size {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            tile_frame_buffer.as_ptr().add(tile_row*tile_size*4),
                            frame.as_mut_ptr().add(first_pixel_index*4),
                            tile_size,
                        );
                    }
                }
                // Same for the depth buffer.
                for tile_row in 0..tile_size {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            tile_depth_buffer.as_ptr().add(tile_row*tile_size),
                            depth_buffer.as_mut_ptr().add(first_pixel_index),
                            tile_size,
                        );
                    }
                }

                // Go to next tile.
                tile_nb += 1;
            }
        }
    }
}
/// Containts the necessary data to handle a triangle from geometry binned in a tile.
#[derive(Clone, Copy)]
struct BinnedTriangle {
    /// Start index of the triangle within the geometry.
    pub triangle_start: usize,
    /// Minimum x value of the triangle's aabs relative to the tile.
    pub min_x: usize,
    /// Minimum y value of the triangle's aabs relative to the tile.
    pub min_y: usize,
    /// Maximum x value of the triangle's aabs relative to the tile.
    pub max_x: usize,
    /// Maximum y value of the triangle's aabs relative to the tile.
    pub max_y: usize,
}
impl BinnedTriangle {
    /// Create default BinnedTriangle with 0 for every value.
    pub fn new() -> Self {
        BinnedTriangle {
            triangle_start: 0,
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
        }
    }
}
