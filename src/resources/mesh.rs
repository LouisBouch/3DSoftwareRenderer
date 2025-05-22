//! Handles everything related to triangle meshes.
use glam::{DMat4, DVec2, DVec3, DVec4};

/// Contains everything required to render a triangle mesh.
pub struct Mesh {
    /// The id of the texture which is owned by the [`super::texture::TextureCatalog`].
    texture_id: Option<u32>,
    /// The transform that moves the mesh from local to world view.
    world_transfrom: DMat4,
    /// The vertices that make up the mesh.
    vertices: Vec<Vertex>,
    /// The list of indices that define the triangles in the mesh. Each successive 3 idex represent
    /// a triangle.
    triangles: Vec<u32>,
}
impl Mesh {
    /// Constructs a mesh from its fields.
    ///
    /// # Arguments
    ///
    /// * `texture_id` - The id of the texture to use, if any. No texture defaults to an invisible
    /// object.
    /// * `world_transfrom` - The transform to convert the mesh from local to wrodl view.
    /// * `vertices` - The vertices making up the mesh.
    /// * `triangles` - The indices representing the triangle within the mesh. Note that no
    /// verifications are made to ensure validity, so the user must be careful with what they
    /// input.
    pub fn new(texture_id: Option<u32>, world_transfrom: DMat4, vertices: Vec<Vertex>, triangles: Vec<u32>) -> Self {
        Mesh {
            texture_id,
            world_transfrom,
            vertices,
            triangles,
        }
    }
    /// Set a new texture for the mesh.
    ///
    /// # Arguments
    ///
    /// * `texture_id` - New texture for the mesh.
    pub fn set_texture(&mut self, texture_id: Option<u32>) {
        self.texture_id = texture_id;
    }

}
/// Contains the information required for a vertex of a triangle mesh.
pub struct Vertex {
    /// Homogeneous position of the vertex.
    position: DVec4,
    /// UV coordinates of the vertex.
    uv: DVec2,
}
impl Vertex {
    /// Constructs a new Vertex.
    ///
    /// # Arguments
    ///
    /// * `position` - The position in space of the vector.
    /// * `uv` - The UV coordinates of the vertex.
    pub fn new(position: DVec3, uv: DVec2) -> Self {
        Vertex {
            position: DVec4::new(position.x, position.y, position.z, 1.0),
            uv,
        }
    }
}
