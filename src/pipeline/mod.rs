//! Contains everytihng that will be needed to render the scene.

use std::collections::HashMap;

use geometry::Geometry;
use glam::{DVec2, Vec3Swizzles, Vec4Swizzles};

use crate::{graphics::screen::Screen, resources::texture::Texture, scene::Scene};

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
                    self.rasterize(&geometry, screen, textures);
                    //screen.draw() (do it somewhere else?)
                }
            }
            crate::scene::camera::Projection::Orthographic { .. } => {
                todo!("Implement orthographic projection.");
            }
        }
    }
    /// Raterizes the geometry on the screen buffer.
    pub fn rasterize(
        &self,
        geometry: &Geometry,
        screen: &mut Screen,
        textures: &HashMap<u32, Texture>,
    ) {
        let vertices = geometry.vertices();
        let triangles = geometry.triangles();
        let frame = screen.pixels_mut().unwrap().frame_mut();
        let (width, height) = (screen.width(), screen.height());

        // Rasterize each triangle.
        for triangle_index_start in (0..triangles.len()).step_by(3) {
            let (a, b, c) = (
                vertices[triangles[triangle_index_start as usize] as usize].xyz(),
                vertices[triangles[triangle_index_start as usize + 1] as usize].xyz(),
                vertices[triangles[triangle_index_start as usize + 2] as usize].xyz(),
            );
            // Get bounding box of triangle.
            // max values are excluded, while min values are included.
            let (min_x, max_x, min_y, max_y) = Pipeline::triangle_aabs(a.xy(), b.xy(), c.xy());
            // Ensure they don't cross the screen's border.
            let min_x = if min_x > width { width } else { min_x };
            let max_x = if max_x > width { width } else { max_x };
            let min_y = if min_y > height { height } else { min_y };
            let max_y = if max_y > height { height } else { max_y };
            // Rasterize over the bounding box.
            for y in min_y..max_y {
                for x in min_x..max_x {
                    // Get barycentric coordinates.
                    // Check if pixel is inside the triangle.
                    // Check if pixel has already been drawn closer to the screen.
                    // Get the UV coordinates of the pixel.
                    // Given the UV coordinates, get the texture color and draw it.
                }
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
