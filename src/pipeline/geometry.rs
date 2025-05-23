//! Contains the building block of the rendering pipeline. Everyting thing drawn on screen will be
//! related to geometry.
use glam::{DMat4, DVec2, DVec3, DVec4};

use crate::resources::mesh::Mesh;

/// Contains the necessary information to draw shapes on screen.
pub struct Geometry {
    /// The id of the texture.
    texture_id: Option<u32>,
    /// Homogeneous position of the vertices making up the shape.
    positions: Vec<DVec4>,
    /// UV coordinates of the vertices.
    uvs: Vec<DVec2>,
    /// The list of indices that define the triangles in the mesh. Each successive 3 idex represent
    /// a triangle.
    triangles: Vec<u32>,
    /// List of inverted w values from the homogeneous coordinates (in clip space before NDC conversion). Useful for interpolation in
    /// screen coordinates, as 1/w is linear in this space.
    clip_w_inv: Vec<f64>,
}

impl Geometry {
    /// Constructs a new geometry from its fields.
    ///
    /// # Warning
    ///
    /// No verifications are made to ensure validity of the uv
    /// coordinates, position of the vertices and indices of the triangles.
    /// It is up to the user to ensure it.
    pub fn new(
        positions: &Vec<DVec4>,
        uvs: &Vec<DVec2>,
        triangles: &Vec<u32>,
        texture_id: Option<u32>,
    ) -> Self {
        Geometry {
            texture_id: texture_id,
            positions: positions.clone(),
            uvs: uvs.clone(),
            triangles: triangles.clone(),
            clip_w_inv: vec![1.0; positions.len() as usize],
        }
    }
    /// Constructs a new geometry from a mesh
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let triangles = mesh.triangles().clone();
        // Populate the vectors.
        for vec in mesh.vertices() {
            positions.push(*vec.position());
            uvs.push(*vec.uv());
        }
        Geometry {
            texture_id: mesh.texture_id(),
            clip_w_inv: vec![1.0; positions.len() as usize],
            positions,
            uvs,
            triangles,
        }
    }

    /// Transforms the vertices in the geometry by applying a linear transform to it.
    pub fn lin_transform(&mut self, transform: &DMat4) {
        for pos in self.positions.iter_mut() {
            *pos = transform.mul_vec4(*pos);
        }
    }

    /// Divide every position by its perspective value w, which is the fourth value in the position
    /// vector. This is called perspective division and is an important part of the rendering
    /// process that allows us to go from clip space to ndc space.
    pub fn perspective_divide(&mut self) {
        for pos in self.positions.iter_mut() {
            let w = pos[3];
            pos[0] /= w;
            pos[1] /= w;
            pos[2] /= w;
            pos[3] = 1.0;
        }
    }
    /// Give the orientation of the camera, cull every triangle pointing away from it.
    pub fn cull_backface(&mut self, camera_orientation: &DVec3) {}

    /// Clip triangles that are straddling the x=±w, y=±w, or z=±w planes (this defines the view
    /// frustum). This creates new triangles in the process and removes some that are outside the planes.
    pub fn clip_geometry(&mut self) {}
}
// Getters and setters
impl Geometry {
    /// Mutable reference to the positions of the vertices making up the mesh.
    pub fn positions_mut(&mut self) -> &mut Vec<DVec4> {
        &mut self.positions
    }
    /// Mutable reference to the uv coordinates of the vertices making up the mesh.
    pub fn uvs_mut(&mut self) -> &mut Vec<DVec2> {
        &mut self.uvs
    }
    /// Mutable reference to the triangles making up the mesh.
    pub fn triangles_mut(&mut self) -> &mut Vec<u32> {
        &mut self.triangles
    }
    /// Mutable reference to the inverted w of the homogeneous coordinate of the vertices making up the mesh.
    ///
    /// This value is used when linearly interpolating coordinates in screen space.
    pub fn clip_w_inv_mut(&mut self) -> &mut Vec<f64> {
        &mut self.clip_w_inv
    }
    /// Gets the texture id if there is one.
    pub fn texture_id(&self) -> Option<u32> {
        self.texture_id
    }
}
