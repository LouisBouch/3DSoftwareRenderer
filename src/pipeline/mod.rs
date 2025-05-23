//! Contains everytihng that will be needed to render the scene.

use std::collections::HashMap;

use geometry::Geometry;

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
        for mesh in scene.meshes() {
            let mut geometry = Geometry::from_mesh(mesh);
            // Convert geometry to world coordinates.
            geometry.lin_transform(mesh.transform());
            // Convert geometry to view space.
            geometry.lin_transform(camera.transform());
            // Do backface culling.
            geometry.cull_backface(&camera.camera_orientation());
            // Convert to clip space.
            geometry.lin_transform(&camera.perspective_transform());
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
    /// Raterizes the geometry on the screen buffer.
    pub fn rasterize(&self, geometry: &Geometry, screen: &Screen, textures: &HashMap<u32, Texture>) {}
}
