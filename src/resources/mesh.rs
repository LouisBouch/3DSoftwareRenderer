//! Handles everything related to triangle meshes.
use glam::{DMat4, DQuat, DVec2, DVec3, DVec4};

/// Contains everything required to render a triangle mesh.
pub struct Mesh {
    /// The id of the texture which is owned by the [`super::texture::TextureCatalog`].
    texture_id: Option<u32>,
    /// Vector defining the mesh's translation.
    translation: DVec3,
    /// Vector defining scaling. (x_scale, y_scale, z_scale)
    scale: DVec3,
    /// Quaternion defining the rotation of the mesh.
    quat: DQuat,
    /// The transform that moves the mesh from local to world view.
    world_transfrom: DMat4,
    /// The local vertices that make up the mesh.
    local_vertices: Vec<Vertex>,
    /// The list of indices that define the triangles in the mesh. Each successive 3 idex represent
    /// a triangle.
    triangles: Vec<u32>,
}
impl Mesh {
    /// Creates a new [`Mesh`].
    ///
    /// # Arguments
    ///
    /// * `texture_id` - The id of the texture to use, if any.
    /// No texture defaults to a black texture.
    /// object.
    /// * `vertices` - The local vertices making up the mesh.
    /// * `triangles` - The indices representing the triangle within the mesh. (The triangles are
    /// defined CCW when looked at from the exterior)
    ///
    /// # Warning
    ///
    /// No verifications are made to ensure validity of the uv
    /// coordinates, position of the vertices and indices of the triangles.
    /// It is up to the user to ensure it.
    pub fn new(texture_id: Option<u32>, vertices: Vec<Vertex>, triangles: Vec<u32>) -> Self {
        Mesh {
            texture_id,
            world_transfrom: DMat4::IDENTITY,
            translation: DVec3::ZERO,
            quat: DQuat::IDENTITY,
            scale: DVec3::new(1.0, 1.0, 1.0),
            local_vertices: vertices,
            triangles,
        }
    }
    /// Given a transformation matrix, apply it to the [`Mesh`].
    pub fn apply_transform(&mut self, transform: &DMat4) {
        self.world_transfrom = *transform * self.world_transfrom;
    }
    /// Translate the mesh in space.
    pub fn translate(&mut self, translation: DVec3) {
        self.translation += translation;
        self.update_transform();
    }
    /// Rotates the camera by a quaternion.
    ///
    /// # Arguments
    ///
    /// * `rot` - The rotation to add to our current rotation.
    pub fn rotate(&mut self, rot: &DQuat) {
        //q_total = q_second * q_first
        self.quat = rot.mul_quat(self.quat).normalize();
        // Ensure the transformation matrix stays up to date.
        self.update_transform();
    }
    /// Scales the mesh multiplicatively with the current scaling.
    pub fn scale_mesh(&mut self, scale: DVec3) {
        self.scale *= scale;
        self.update_transform();
    }
    /// Scales the mesh additively with the current scaling.
    pub fn scale_add(&mut self, scale: DVec3) {
        self.scale += scale;
        self.update_transform();
    }
    /// Update the transform of the mesh to keep it synchronized with its
    /// translation, rotation and scale.
    fn update_transform(&mut self) {
        self.world_transfrom = DMat4::from_translation(self.translation) * DMat4::from_quat(self.quat) * DMat4::from_scale(self.scale);
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
    /// Sets the mesh rotation with a quaternion.
    ///
    /// # Arguments
    ///
    /// * `rot` - The rotation to set our camera to.
    pub fn set_quat(&mut self, quat: &DQuat) {
        self.quat = (*quat).normalize();
        // Ensure the transformation matrix stays up to date.
        self.update_transform();
    }
    /// Sets the mesh scaling.
    pub fn set_scale(&mut self, scale: DVec3) {
        self.scale = scale;
        // Ensure the transformation matrix stays up to date.
        self.update_transform();
    }
    /// Sets the mesh translation.
    pub fn set_translation(&mut self, translation: DVec3) {
        self.translation = translation;
        // Ensure the transformation matrix stays up to date.
        self.update_transform();
    }
    /// Gets the position of the mesh.
    pub fn translation(&self) -> &DVec3 {
        &self.translation
    }
    /// Gets the scale of the mesh.
    pub fn scale(&self) -> &DVec3 {
        &self.scale
    }
    /// Gets the quaternion of the mesh.
    pub fn quat(&self) -> &DQuat {
        &self.quat
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
