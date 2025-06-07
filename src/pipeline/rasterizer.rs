//! Contains everything that will be needed to rasterize an image.
use core::f64;

use glam::{DVec2, Vec3Swizzles, Vec4Swizzles};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{algorithm, graphics::screen::Screen, resources::texture::Texture};

use super::{geometry::Geometry, BinnedTriangle};

/// Holds the necessary values for rasterizing.
pub struct Rasterizer {
    /// Divides the screen into tiles of size [`tile_size`].
    tile_size: usize,
    /// The depth and pixel buffer for each tile on the screen.
    tiles: Vec<Tile>,
}
impl Rasterizer {
    /// Create a new rasterizer.
    ///
    /// # Arguments
    ///
    /// * `tile_size` - Size of the tiles the rasterizer will split the screen with.
    /// * `width` - Width of the screen the rasterizer will draw on.
    /// * `Height` - Height of the screen the rasterizer will draw on.
    pub fn new(tile_size: usize, width: usize, height: usize) -> Self {
        // Figure out how many tiles are required given its size.
        let (nb_tiles_x, nb_tiles_y) = (
            (width + tile_size - 1) / tile_size,
            (height + tile_size - 1) / tile_size,
        );
        // Initialize the tiles to be transparent black at every pixel with maximum depth.
        let tiles = vec![
            Tile {
                depth_buf: vec![f64::INFINITY; tile_size * tile_size],
                frame_buf: vec![0; tile_size * tile_size * 4]
            };
            nb_tiles_x * nb_tiles_y
        ];
        Rasterizer { tile_size, tiles }
    }
    /// Clears the tiles of the rasterizer.
    /// TODO: Add dirty tile system and only fill these up.
    pub fn clear(&mut self) {
        for tile in self.tiles.iter_mut() {
            tile.depth_buf.fill(f64::INFINITY);
            // tile.frame_buf.fill(0);
        }
    }
    /// Raterizes the geometry on the screen buffer while making use of multithreading.
    ///
    /// Uses a tilling approach, where the screen is divided into different
    /// tiles of size `tile_size`² and each one is rasterized by a different using rayon.
    /// TILES GO LEFT TO RIGHT, TOP TO BOTTOM (Row major).
    pub fn rasterize_threaded(
        &mut self,
        geometry: &Geometry,
        screen: &mut Screen,
        texture: Option<&Texture>,
    ) {
        let tile_size = self.tile_size();
        // Get useful values for rasterizing.
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

        // Figure out how many tiles are required given its size.
        let (nb_tiles_x, nb_tiles_y) = (
            (width + tile_size - 1) / tile_size,
            (height + tile_size - 1) / tile_size,
        );

        // Create tile buffers for the f        // Bin the triangles into the different tiles.
        let mut binned_triangles: Vec<Vec<BinnedTriangle>> =
            vec![Vec::new(); nb_tiles_x * nb_tiles_y];

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
            // Both first and last tiles are inclusive.
            let (first_tile_x, last_tile_x, first_tile_y, last_tile_y) = (
                min_x / tile_size,
                max_x / tile_size,
                min_y / tile_size,
                max_y / tile_size,
            );
            // Add the triangle to the bin of each tile.
            for tile_y in first_tile_y..=last_tile_y {
                for tile_x in first_tile_x..=last_tile_x {
                    let mut binned_triangle = BinnedTriangle::new();
                    // Get the relative position of the aabs within the tile.
                    binned_triangle.min_x = min_x - (tile_x * tile_size).min(min_x);
                    binned_triangle.min_y = min_y - (tile_y * tile_size).min(min_y);
                    binned_triangle.max_x = (max_x - tile_x * tile_size).min(tile_size - 1);
                    binned_triangle.max_y = (max_y - tile_y * tile_size).min(tile_size - 1);
                    binned_triangle.triangle_start = triangle_index_start;
                    // Push it in the corresponding bin.
                    binned_triangles[tile_x + tile_y * nb_tiles_x].push(binned_triangle);
                }
            }
        }
        // Rasterize in parallel on each tile.
        // frame_buffers
        //     .par_iter_mut()
        //     .zip(depth_buffers.par_iter_mut())
        self.tiles_mut()
            .par_iter_mut()
            .enumerate()
            .for_each(|(tile_nb, tile)| {
                let (tile_frame_buffer, tile_depth_buf) = tile.get_buffers();
                // Obtain the tile's coordinate from the tile number.
                let x_offset = (tile_nb % nb_tiles_x) * tile_size;
                let y_offset = (tile_nb / nb_tiles_x) * tile_size;

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

                    // Screen coordinates at the top left of the aabs.
                    // Add 0.5 to both x and y in order to sample the pixel at its center.
                    let min_posf64_screen = DVec2::new(
                        (x_offset + min_x) as f64 + 0.5,
                        (y_offset + min_y) as f64 + 0.5,
                    );

                    // Get barycentric coordinates at min_pos.
                    let (alpha_00, beta_00, gamma_00) = (
                        alpha_grad.dot(min_posf64_screen - c.xy()),
                        beta_grad.dot(min_posf64_screen - a.xy()),
                        gamma_grad.dot(min_posf64_screen - b.xy()),
                    );
                    // Initialize coordinates for first row (redundant, but clearer)
                    let (mut alpha_0y, mut beta_0y, mut gamma_0y) = (alpha_00, beta_00, gamma_00);

                    // Rasterize over the bounding box (with respect to the tile).
                    for y in min_y..=max_y {
                        let mut pixel_index = min_x + y * tile_size; // With respect to the tile.
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
                            // &&
                            // Make sure pixels closer to the screen have not been been drawn.
                            // Smaller depth means closer to screen.
                            if ((alpha_xy >= 0.0) & (beta_xy >= 0.0) & (gamma_xy >= 0.0))
                                && depth < tile_depth_buf[pixel_index]
                            {
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
                                                nb_channels,
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
        // Write back to the main frame buffer.
        let frame = screen.pixels_mut().unwrap().frame_mut();
        let tiles = self.tiles_mut();
        for tile_y in 0..nb_tiles_y {
            let mut tile_nb = tile_y * nb_tiles_x;
            for tile_x in 0..nb_tiles_x {
                // Get references to the buffers.
                let tile = &tiles[tile_nb];
                let tile_frame_buffer: &[u8] = &tile.frame_buf;

                // Get pixel offset.
                let first_pixel_index = tile_x * tile_size + tile_y * tile_size * width;

                // For each row in the tile, copy it over to the main frame buffer.
                for tile_row in 0..tile_size {
                    // Check if you are going beyond the screen's height.
                    if tile_row + tile_y * tile_size >= height {
                        break;
                    }
                    // Ensure you don't copy too far on the screen and end up
                    // wrapping/out of bounds.
                    let pixels_to_copy = (width - tile_x * tile_size).min(tile_size);
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            tile_frame_buffer.as_ptr().add(tile_row * tile_size * 4),
                            frame
                                .as_mut_ptr()
                                .add((first_pixel_index + tile_row * width) * 4),
                            pixels_to_copy * 4,
                        );
                    }
                }
                // Go to next tile.
                tile_nb += 1;
            }
        }
    }
}
// Getters and setters
impl Rasterizer {
    // Gets the tile_size of the rasterizer.
    pub fn tile_size(&self) -> usize {
        self.tile_size
    }
    // Mutable reference to the tiles.
    pub fn tiles_mut(&mut self) -> &mut [Tile] {
        &mut self.tiles
    }
}
/// Pixel and depth buffer for a single tile.
#[derive(Clone)]
pub struct Tile {
    /// The depth buffer for a tile on the screen.
    pub depth_buf: Vec<f64>,
    /// The frame/pixel buffer for a tile one the screen.
    pub frame_buf: Vec<u8>,
}
impl Tile {
    pub fn get_buffers(&mut self) -> (&mut [u8], &mut [f64]) {
        (&mut self.frame_buf, &mut self.depth_buf)
    }
}
