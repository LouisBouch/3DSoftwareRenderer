//! Contains everytihng that will be needed to render the scene.

use geometry::Geometry;
use rasterizer::Rasterizer;
use shader::Shader;

use crate::{graphics::screen::Screen, scene::Scene};

pub mod geometry;
mod rasterizer;
mod transforms;
pub mod shader;

/// Contains values imprtant for rendering.
pub struct Pipeline {
    rasterizer: rasterizer::Rasterizer,
    shader: Shader,
}

impl Pipeline {
    /// Constructs a pipeline.
    ///
    /// # Arguments
    ///
    /// * `tile_size` - Size of the tiles the rasterizer will split the screen with.
    /// * `width` - Width of the screen the pipeline will draw on.
    /// * `height` - Height of the screen the pipeline will draw on.
    /// * `shader` - What type of shader to use in the pipeline.
    pub fn new(tile_size: usize, width: usize, height: usize, shader: Shader) -> Self {
        Pipeline {
            rasterizer: Rasterizer::new(tile_size, width, height),
            shader: shader,
        }
    }
    /// Clear rasterizer and others values before processing the scene again.
    pub fn clear(&mut self, color: &[u8]) {
        self.rasterizer.clear_with_color(color);
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
                    // Set important values for rasterization.
                    geometry.set_clip_w_inv();
                    // let clip_to_world = (perspective_transform*camera_inv_transform).inverse();
                    let clip_to_world = (perspective_transform*camera_inv_transform).inverse();
                    geometry.set_triangle_world_normals(clip_to_world);
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
                    self.rasterizer.rasterize_threaded(&geometry, screen, texture, &self.shader, scene.lights());
                }
            }
            crate::scene::camera::Projection::Orthographic { .. } => {
                todo!("Implement orthographic projection.");
            }
        }
    }
}
