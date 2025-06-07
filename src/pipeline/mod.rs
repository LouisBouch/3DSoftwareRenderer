//! Contains everytihng that will be needed to render the scene.

use geometry::Geometry;
use rasterizer::Rasterizer;

use crate::{graphics::screen::Screen, scene::Scene};

pub mod geometry;
mod rasterizer;
mod transforms;

/// Contains values imprtant for rendering.
pub struct Pipeline {
    rasterizer: rasterizer::Rasterizer,
}

impl Pipeline {
    /// Constructs a pipeline.
    ///
    /// # Arguments
    ///
    /// * `tile_size` - Size of the tiles the rasterizer will split the screen with.
    /// * `width` - Width of the screen the pipeline will draw on.
    /// * `Height` - Height of the screen the pipeline will draw on.
    pub fn new(tile_size: usize, width: usize, height: usize) -> Self {
        Pipeline {
            rasterizer: Rasterizer::new(tile_size, width, height),
        }
    }
    /// Clear rasterizer and others values before processing the scene again.
    pub fn clear(&mut self) {
        self.rasterizer.clear();
    }
    /// Processes the data contained within the scene and prepares it for rendering.
    ///
    /// # Arguments
    ///
    /// * `scene` - The scene that will be processed. Every mesh withing will be rendererd.
    /// * `screen` - Where the scene will be rasterized.
    pub fn process_scene(&mut self, scene: &Scene, screen: &mut Screen) {
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
                    self.rasterizer.rasterize_threaded(&geometry, screen, texture);
                }
            }
            crate::scene::camera::Projection::Orthographic { .. } => {
                todo!("Implement orthographic projection.");
            }
        }
    }
}
/// Containts the necessary data to handle a triangle from geometry binned in a tile.
#[derive(Clone, Copy)]
struct BinnedTriangle {
    /// Start index of the triangle within the [`geometry::Geometry`]'s triangles vector.
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
