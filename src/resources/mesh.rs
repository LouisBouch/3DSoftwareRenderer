//! Handles everything related to triangle meshes.
use glam::{DMat4, DVec2, DVec3, DVec4};

/// Contains everything required to render a triangle mesh.
pub struct Mesh {
    /// The id of the texture which is owned by the [`super::texture::TextureCatalog`].
    texture_id: Option<u32>,
    /// The transform that moves the mesh from local to world view.
    world_transfrom: DMat4,
    /// The local vertices that make up the mesh.
    local_vertices: Vec<Vertex>,
    /// The list of indices that define the triangles in the mesh. Each successive 3 idex represent
    /// a triangle.
    triangles: Vec<u32>,
}
impl Mesh {
    /// Creates a new [`Mesh`] from its fields.
    ///
    /// # Arguments
    ///
    /// * `texture_id` - The id of the texture to use, if any. No texture defaults to an invisible
    /// object.
    /// * `world_transfrom` - The transform to convert the mesh from local to wrodl view.
    /// * `vertices` - The local vertices making up the mesh.
    /// * `triangles` - The indices representing the triangle within the mesh. (The triangles are
    /// defined CCW when looked at from the exterior)
    ///
    /// # Warning
    ///
    /// No verifications are made to ensure validity of the uv
    /// coordinates, position of the vertices and indices of the triangles.
    /// It is up to the user to ensure it.
    pub fn new(
        texture_id: Option<u32>,
        world_transfrom: DMat4,
        vertices: Vec<Vertex>,
        triangles: Vec<u32>,
    ) -> Self {
        Mesh {
            texture_id,
            world_transfrom,
            local_vertices: vertices,
            triangles,
        }
    }
    /// Given a transformation matrix, apply it to the [`Mesh`].
    pub fn apply_transform(&mut self, transform: &DMat4) {
        self.world_transfrom *= *transform;
    }
}
// Getters and setters
impl Mesh {
    /// Set a new texture for the mesh.
    ///
    /// # Arguments
    ///
    /// * `texture_id` - New texture for the mesh.
    pub fn set_texture(&mut self, texture_id: Option<u32>) {
        self.texture_id = texture_id;
    }
    /// Gets the texture id if there is one.
    pub fn texture_id(&self) -> Option<u32> {
        self.texture_id
    }
    /// Exposes a reference to the list of vertices making up the mesh.
    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.local_vertices
    }
    /// Exposes a reference to the list of triangles making up the mesh.
    pub fn triangles(&self) -> &Vec<u32> {
        &self.triangles
    }
    /// Exposes a reference to the transform which converts the mesh from local to world space.
    pub fn transform(&self) -> &DMat4 {
        &self.world_transfrom
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
// Getters and setters
impl Vertex {
    /// Exposes a reference to the homogeneous position of the vertex in space.
    pub fn position(&self) -> &DVec4 {
        &self.position
    }
    /// Exposes a reference to the UV coordinate of the vertex.
    pub fn uv(&self) -> &DVec2 {
        &self.uv
    }
}
