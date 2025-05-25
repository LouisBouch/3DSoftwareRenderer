//! Contains the building block of the rendering pipeline. Everyting thing drawn on screen will be
//! related to geometry.
use std::collections::HashMap;

use glam::{usize, DMat4, DVec2, DVec3, DVec4, Vec4Swizzles};
use ndarray::array;

use crate::{algorithm, resources::mesh::Mesh};

/// Contains the necessary information to draw shapes on screen.
pub struct Geometry {
    /// The id of the texture.
    texture_id: Option<u32>,
    /// Homogeneous position of the vertices making up the shape.
    vertices: Vec<DVec4>,
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
        vertices: &Vec<DVec4>,
        uvs: &Vec<DVec2>,
        triangles: &Vec<u32>,
        texture_id: Option<u32>,
    ) -> Self {
        Geometry {
            texture_id: texture_id,
            vertices: vertices.clone(),
            uvs: uvs.clone(),
            triangles: triangles.clone(),
            clip_w_inv: vec![1.0; vertices.len() as usize],
        }
    }
    /// Constructs a new geometry from a mesh
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let mut vertices = Vec::new();
        let mut uvs = Vec::new();
        let triangles = mesh.triangles().clone();
        // Populate the vectors.
        for vec in mesh.vertices() {
            vertices.push(*vec.position());
            uvs.push(*vec.uv());
        }
        Geometry {
            texture_id: mesh.texture_id(),
            clip_w_inv: vec![1.0; vertices.len() as usize],
            vertices,
            uvs,
            triangles,
        }
    }

    /// Transforms the vertices in the geometry by applying a linear transform to it.
    pub fn lin_transform(&mut self, transform: &DMat4) {
        for pos in self.vertices.iter_mut() {
            *pos = transform.mul_vec4(*pos);
        }
    }

    /// Divide every position by its perspective value w, which is the fourth value in the position
    /// vector. This is called perspective division and is an important part of the rendering
    /// process that allows us to go from clip space to ndc space.
    pub fn perspective_divide(&mut self) {
        for pos in self.vertices.iter_mut() {
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
            let (ai, bi, ci) = (
                self.triangles[triangle_index_start],
                self.triangles[triangle_index_start + 1],
                self.triangles[triangle_index_start + 2],
            );
            // The three triangle vertices.
            let (a, b, c) = (
                self.vertices[ai as usize].xyz(),
                self.vertices[bi as usize].xyz(),
                self.vertices[ci as usize].xyz(),
            );
            // Vector from camera to first vertex of triangle.
            let cam_to_tri = a - camera_position;
            // Vector normal to the triangle pointing to the exterior of the mesh.
            let tri_face_normal = (b - a).cross(c - a);
            // If the triangle is pointing towards the camera, keep it.
            if cam_to_tri.dot(tri_face_normal) < 0.0 {
                triangles.push(ai);
                triangles.push(bi);
                triangles.push(ci);
            }
        }
        self.triangles = triangles;
    }

    /// Clip triangles that are straddling the x=±w, y=±w, or z=±w planes (this defines the view
    /// frustum). This creates new triangles in the process and removes some that are outside the planes.
    /// Uses the sutherland-hodgman polygon clipping algorithm.
    pub fn clip_geometry(&mut self) {
        // Create a new list of triangles that are created during the clipping, or survive it.
        let mut triangles = Vec::<u32>::with_capacity(self.triangles.len());
        // The various clipping plane defined for the frustum.
        #[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
        enum ClipPlane {
            XP,
            XN,
            YP,
            YN,
            ZP,
            ZN,
        }
        // Check each triangle within the mesh and clip those straddling the frustum and remove
        // those outside of it.
        //
        // Quick proof to show the hyperplane normal points outside the frustum.
        //
        // We know that -w<=x<=w. Thus, for the hyperplane x=w, values inside the frustum require x<w.
        // Also, a point p will be on the positive side of the hyperplane (same direction as normal) when
        // n.(p-p_0) > 0, and negative side otherwise.
        // Given that a point within the frustum (in x) will yield
        // n.(p-p_0) = n.p = x - w < 0, we have that it is in the negative direction.
        // This means the normal must be pointing outside the frustum.
        let hyperplanes = vec![
            (ClipPlane::XP, DVec4::new(1.0, 0.0, 0.0, -1.0)),
            (ClipPlane::XN, DVec4::new(1.0, 0.0, 0.0, 1.0)),
            (ClipPlane::YP, DVec4::new(0.0, 1.0, 0.0, -1.0)),
            (ClipPlane::YN, DVec4::new(0.0, 1.0, 0.0, 1.0)),
            (ClipPlane::ZP, DVec4::new(0.0, 0.0, 1.0, -1.0)),
            (ClipPlane::ZN, DVec4::new(0.0, 0.0, 1.0, 1.0)),
        ];
        // A cache that remembers which planes intersected with which edges and at which point.
        let mut intersection_cache: HashMap<(u32, u32, ClipPlane), u32> = HashMap::new();
        for triangle_index_start in (0..self.triangles.len()).step_by(3) {
            // Get the vertex indice corresponding to the triangle.
            let indices = vec![
                self.triangles[triangle_index_start],
                self.triangles[triangle_index_start + 1],
                self.triangles[triangle_index_start + 2],
            ];
            // List of vertex indices making up the new shape after clipping.
            let mut shape: Vec<u32> = Vec::new();
            // Clip the triangle against the 6 clipping planes (x=±w, y=±w, and z=±w).
            for (plane_type, plane) in hyperplanes.iter() {
                // Check whether the edge straddles the plane.
                for edge in 0..3 {
                    let ai = indices[edge];
                    let bi = indices[(edge + 1) % 3];
                    // The vertex positions of the edge.
                    let a = self.vertices[ai as usize];
                    let b = self.vertices[bi as usize];
                    // Check whether a and b are inside or outside the plane.
                    let a_in = a.dot(*plane) <= 0.0;
                    let b_in = b.dot(*plane) <= 0.0;

                    // Sutherland-Hodgman algorithm.
                    if (b_in && !a_in) || (a_in && !b_in) {
                        // Here b is inside but not a, or a is inside but not b.
                        let Some(t) =
                            algorithm::lin_plane_intersect4(DVec4::ZERO, *plane, a, b - a)
                        // If t is parallel to the plane, add b when it is outside or both if a is
                        // outside.
                        else {
                            if !b_in {
                                shape.push(bi);
                            } else {
                                shape.push(ai);
                                shape.push(bi);
                            }
                            println!("Parallel issue: Sutherland-Hodgman");
                            continue;
                        };
                        // Order the edges such that e1 < e2
                        let (mut e1, mut e2) = (ai, bi);
                        if ai > bi {
                            (e1, e2) = (bi, ai);
                        }
                        // Check where this edge already has a computed intersection.
                        if let Some(&ci) = intersection_cache.get(&(e1, e2, *plane_type)) {
                            shape.push(ci);
                        } else {
                            // Add the intersection to the list.
                            let c = a.lerp(b, t);
                            let uv = self.uvs[ai as usize].lerp(self.uvs[bi as usize], t);
                            let ci = self.vertices.len() as u32;
                            self.vertices.push(c);
                            self.uvs.push(uv);
                            intersection_cache.insert((e1, e2, *plane_type), ci);
                        }
                    }
                    if b_in {
                        // Here b is inside.
                        shape.push(bi);
                    }
                }
            }
            // Triangulate the shape.
            for v in 0..(shape.len() - 2) {
                triangles.push(shape[v]);
                triangles.push(shape[v + 1]);
                triangles.push(shape[v + 2]);
            }
        }
        self.triangles = triangles;
    }
}
// Getters and setters
impl Geometry {
    /// Mutable reference to the positions of the vertices making up the mesh.
    pub fn vertices_mut(&mut self) -> &mut Vec<DVec4> {
        &mut self.vertices
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
