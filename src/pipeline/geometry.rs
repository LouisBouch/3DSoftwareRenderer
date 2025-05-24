//! Contains the building block of the rendering pipeline. Everyting thing drawn on screen will be
//! related to geometry.
use glam::{DMat4, DVec2, DVec3, DVec4, Vec4Swizzles};

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
    /// a triangle. (Defined CCW)
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
    /// Given the position of the camera, cull every triangle pointing away from it.
    ///
    /// * `camera_position` - The camera position in world space.
    pub fn cull_backface(&mut self, camera_position: &DVec3) {
        // Create a new list of triangles which are facing towards the camera.
        let mut triangles = Vec::<u32>::with_capacity(self.triangles.len());
        // Check each triangle within the mesh and only keep those pointing towards the camera.
        for triangle_index_start in (0..self.triangles.len()).step_by(3) {
            // Get the vertex indice corresponding to the triangle.
            let indices = (
                self.triangles[triangle_index_start],
                self.triangles[triangle_index_start + 1],
                self.triangles[triangle_index_start + 2],
            );
            // The three triangle vertices.
            let (a, b, c) = (
                self.positions[indices.0 as usize].xyz(),
                self.positions[indices.1 as usize].xyz(),
                self.positions[indices.2 as usize].xyz(),
            );
            // Vector from camera to first vertex of triangle.
            let cam_to_tri = a - camera_position;
            // Vector normal to the triangle pointing to the exterior of the mesh.
            let tri_face_normal = (b-a).cross(c-a);
            // If the triangle is pointing towards the camera, keep it.
            if cam_to_tri.dot(tri_face_normal) < 0.0 {
                triangles.push(indices.0);
                triangles.push(indices.1);
                triangles.push(indices.2);
            }
        }
        self.triangles = triangles;
    }

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
