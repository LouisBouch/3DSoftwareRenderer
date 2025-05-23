//! Contains everything needed to render the environment.

use camera::Camera;

use crate::resources::{mesh::Mesh, texture::TextureCatalog};

/// Contains everything in the scene that will be rendered.
pub struct Scene {
    /// The camera from which the environment will be seen.
    camera: Camera,
    /// The list of textures in use in the scene.
    texture_catalog: TextureCatalog,
    /// A list of meshes inside the scene.
    meshes: Vec<Mesh>,
}
impl Scene {
    /// Create new scene with default camera placement.
    pub fn new() -> Self {
        let camera = Camera::default();
        Scene { camera, texture_catalog: TextureCatalog::new(), meshes: Vec::new() }
    }
    /// Adds a mesh to the scene.
    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }
}
// Getters and setters.
impl Scene {
    /// Mutable reference for the camera.
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
    /// Reference for the camera.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    /// Mutable reference for the texture catalog.
    pub fn texture_catalog_mut(&mut self) -> &mut TextureCatalog {
        &mut self.texture_catalog
    }
    /// Reference for the texture catalog.
    pub fn texture_catalog(&self) -> &TextureCatalog {
        &self.texture_catalog
    }
    /// Mutable reference for the mesh vector.
    pub fn meshes_mut(&mut self) -> &mut Vec<Mesh> {
        &mut self.meshes
    }
    /// Reference for the mesh vector.
    pub fn meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }
}
pub mod camera;
mod light;

