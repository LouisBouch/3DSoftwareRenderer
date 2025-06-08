//! Contains the building block of the rendering pipeline. Everyting thing drawn on screen will be
//! related to geometry.
use std::collections::HashMap;

use glam::{usize, DMat4, DVec2, DVec3, DVec4, Vec4Swizzles};

use crate::{algorithm, resources::mesh::Mesh};

/// Contains the necessary information to draw shapes on screen.
#[derive(Clone)]
pub struct Geometry {
    /// The id of the texture.
    texture_id: Option<u32>,
    /// Homogeneous position of the vertices making up the shape.
    vertices: Vec<DVec4>,
    /// UV coordinates of the vertices.
    uvs: Vec<DVec2>,
    /// The list of indices that define the triangles in the mesh. Each successive 3 idex represent
    /// a triangle. (Defined CCW)
    triangles: Vec<usize>,
    /// List of inverted w values from the homogeneous coordinates (in clip space before NDC conversion). Useful for interpolation in
    /// screen coordinates, as 1/w is linear in this space.
    clip_w_inv: Vec<f64>,
    /// List of normals for each triangle when in world space.
    triangle_normals: Vec<DVec3>,
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
        triangles: &Vec<usize>,
        texture_id: Option<u32>,
    ) -> Self {
        Geometry {
            texture_id: texture_id,
            vertices: vertices.clone(),
            uvs: uvs.clone(),
            triangles: triangles.clone(),
            clip_w_inv: vec![1.0; vertices.len()],
            triangle_normals: Vec::with_capacity(vertices.len() / 3),
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
        let nb_triangles = vertices.len() / 3;
        Geometry {
            texture_id: mesh.texture_id(),
            clip_w_inv: Vec::new(),
            vertices,
            uvs,
            triangles,
            triangle_normals: Vec::with_capacity(nb_triangles),
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
    /// Needs to be called when the geoemtry is in world space. If in camera space, use
    /// camera_position = 0.
    ///
    /// * `camera_position` - The camera position in world space.
    pub fn cull_backface(&mut self, camera_position: &DVec3) {
        // Create a new list of triangles which are facing towards the camera.
        let mut triangles = Vec::<usize>::with_capacity(self.triangles.len());
        // Check each triangle within the mesh and only keep those pointing towards the camera.
        for triangle_index_start in (0..self.triangles.len()).step_by(3) {
            // Get the vertex indices corresponding to the triangle.
            let (ai, bi, ci) = (
                self.triangles[triangle_index_start],
                self.triangles[triangle_index_start + 1],
                self.triangles[triangle_index_start + 2],
            );
            // The three triangle vertices.
            let (a, b, c) = (
                self.vertices[ai].xyz(),
                self.vertices[bi].xyz(),
                self.vertices[ci].xyz(),
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
        let mut triangles = Vec::<usize>::with_capacity(self.triangles.len());
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
        // We know that -w<=x<=w for x to be in the frustum.
        // Thus, for the hyperplane x-w=0, values inside the frustum require x<w.
        // Also, a point p will be on the positive side of the hyperplane (same direction as normal) when
        // n.(p-p_0) > 0, and negative side otherwise. (p_0 = 0 for our hyperplanes)
        // Given that a point within the frustum (in x) will yield
        // n.(p-p_0) = n.p = x - w < 0, we have that it is in the negative direction when using the
        // plane x=w.
        // This means the normal is pointing outside the frustum.
        //
        // Similarly, for the hyperplane -x-w=0, values inside the frustum require x>-w.
        // Thus, n.(p-p_0) = n.p = -x - w > 0.
        //
        // This shows that using normals [1,0,0,-1] and [-1,0,0,-1] for our hyperplanes give us
        // negative dot products when the point is inside, and positive dot products when the point
        // is outside.
        //
        let hyperplanes = vec![
            (ClipPlane::XP, DVec4::new(1.0, 0.0, 0.0, -1.0)),
            (ClipPlane::XN, DVec4::new(-1.0, 0.0, 0.0, -1.0)),
            (ClipPlane::YP, DVec4::new(0.0, 1.0, 0.0, -1.0)),
            (ClipPlane::YN, DVec4::new(0.0, -1.0, 0.0, -1.0)),
            (ClipPlane::ZP, DVec4::new(0.0, 0.0, 1.0, -1.0)),
            (ClipPlane::ZN, DVec4::new(0.0, 0.0, -1.0, -1.0)),
        ];
        // A cache that remembers which planes intersected with which edges and at which point.
        let mut intersection_cache: HashMap<(usize, usize, ClipPlane), usize> = HashMap::new();
        for triangle_index_start in (0..self.triangles.len()).step_by(3) {
            // Get the vertex indices corresponding to the triangle. Make it the current shape.
            let mut shape = vec![
                self.triangles[triangle_index_start],
                self.triangles[triangle_index_start + 1],
                self.triangles[triangle_index_start + 2],
            ];
            // Clip the triangle against the 6 clipping planes (x=±w, y=±w, and z=±w).
            for (plane_type, plane_n) in hyperplanes.iter() {
                // List of vertex indices making up the new shape after clipping.
                let mut new_shape: Vec<usize> = Vec::new();
                // Check whether the edges straddle the plane.
                for edge in 0..shape.len() {
                    let ai = shape[edge];
                    let bi = shape[(edge + 1) % shape.len()];
                    // let ai = shape[(edge + shape.len() - 1) % shape.len()];
                    // let bi = shape[edge];
                    // The vertex positions of the edge.
                    let a = self.vertices[ai];
                    let b = self.vertices[bi];
                    // Check whether a and b are inside or outside the plane.
                    let a_in = plane_n.dot(a) <= 0.0;
                    let b_in = plane_n.dot(b) <= 0.0;

                    // Sutherland-Hodgman algorithm.
                    if (b_in && !a_in) || (a_in && !b_in) {
                        // Here b is inside but not a, or a is inside but not b.
                        let Some(t) =
                            algorithm::lin_plane_intersect4(DVec4::ZERO, *plane_n, a, b - a)
                        // If t is parallel to the plane, add b when it is outside or both if a is
                        // outside.
                        else {
                            if !b_in {
                                new_shape.push(bi);
                            } else {
                                new_shape.push(ai);
                                new_shape.push(bi);
                            }
                            println!("Parallel issue: Sutherland-Hodgman");
                            continue;
                        };
                        // Order the edges such that e1 < e2
                        let (mut e1, mut e2) = (ai, bi);
                        if ai > bi {
                            (e1, e2) = (bi, ai);
                        }
                        // Check whether this edge already has a computed intersection.
                        if let Some(&ci) = intersection_cache.get(&(e1, e2, *plane_type)) {
                            new_shape.push(ci);
                        } else {
                            // Add the intersection to the geometry.
                            // TODO: Don't add it directly to the geometry, as some intersections
                            // are later removed through other plane clipping.
                            let c = a.lerp(b, t);
                            self.vertices.push(c);

                            let uv = self.uvs[ai].lerp(self.uvs[bi], t);
                            self.uvs.push(uv);

                            // And add it to the new shape.
                            let ci = self.vertices.len() - 1;
                            intersection_cache.insert((e1, e2, *plane_type), ci);
                            new_shape.push(ci);
                        }
                    }
                    if b_in {
                        // Here b is inside.
                        new_shape.push(bi);
                    }
                }
                shape = new_shape;
            }
            // Triangulate the shape, if it exists.
            if shape.len() >= 3 {
                let fan_base = shape[0];
                for v in 1..(shape.len() - 1) {
                    triangles.push(fan_base);
                    triangles.push(shape[v]);
                    triangles.push(shape[v + 1]);
                }
            }
        }
        self.triangles = triangles;
    }
    /// Uses the current w value to create the `clip_w_inv` values. Just does 1/w.
    ///
    /// This method is called when we enter clip space, as the 1/w at this point is linear in ndc
    /// space, and used for interpolation of uv coordinates.
    pub fn set_clip_w_inv(&mut self) {
        self.clip_w_inv.clear();
        for vertex in self.vertices.iter() {
            self.clip_w_inv.push(1.0 / vertex[3]);
        }
    }
    /// Sets the normals for the triangles when in world space.
    ///
    /// To do so, call this method when the geoemtry has been clipped, but introduce the matrix
    /// that allows to go from clip space to world space. That way the normals are computed as if
    /// they were in world space but with the new clipped triangles.
    pub fn set_triangle_world_normals(&mut self, clip_to_world: DMat4) {
        let triangles = &self.triangles;
        let vertices = &self.vertices;
        for triangle_index_start in (0..triangles.len()).step_by(3) {
            // Triangle vertex indices.
            let (ai, bi, ci) = (
                triangles[triangle_index_start],
                triangles[triangle_index_start + 1],
                triangles[triangle_index_start + 2],
            );
            // Triangle's vertex positions in screen and world space.
            let (a, b, c) = (
                (clip_to_world * vertices[ai]).xyz(),
                (clip_to_world * vertices[bi]).xyz(),
                (clip_to_world * vertices[ci]).xyz(),
            );
            let triangle_normal = (b - a).cross(c - a).normalize();
            self.triangle_normals.push(triangle_normal);
        }
    }
}
// Getters and setters
impl Geometry {
    /// Mutable reference to the positions of the vertices making up the mesh.
    pub fn vertices_mut(&mut self) -> &mut [DVec4] {
        &mut self.vertices
    }
    /// Reference to the positions of the vertices making up the mesh.
    pub fn vertices(&self) -> &[DVec4] {
        &self.vertices
    }
    /// Mutable reference to the uv coordinates of the vertices making up the mesh.
    pub fn uvs_mut(&mut self) -> &mut [DVec2] {
        &mut self.uvs
    }
    /// Reference to the uv coordinates of the vertices making up the mesh.
    pub fn uvs(&self) -> &[DVec2] {
        &self.uvs
    }
    /// Mutable reference to the triangles making up the mesh.
    pub fn triangles_mut(&mut self) -> &mut [usize] {
        &mut self.triangles
    }
    /// Reference to the triangles making up the mesh.
    pub fn triangles(&self) -> &[usize] {
        &self.triangles
    }
    /// Mutable reference to the inverted w of the homogeneous coordinate of the vertices making up the mesh.
    ///
    /// This value is used when linearly interpolating coordinates in screen space.
    pub fn clip_w_inv_mut(&mut self) -> &mut [f64] {
        &mut self.clip_w_inv
    }
    /// Reference to the inverted w of the homogeneous coordinate of the vertices making up the mesh.
    ///
    /// This value is used when linearly interpolating coordinates in screen space.
    pub fn clip_w_inv(&self) -> &[f64] {
        &self.clip_w_inv
    }
    /// Gets the texture id if there is one.
    pub fn texture_id(&self) -> Option<u32> {
        self.texture_id
    }
    /// The normals for the triangles if they were in world space.
    pub fn triangle_normals(&self) -> &[DVec3] {
        &self.triangle_normals
    }
}
