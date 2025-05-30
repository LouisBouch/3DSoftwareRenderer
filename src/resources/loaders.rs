//! Handles the loading of all ressources.

use glam::{DVec2, DVec3};

use crate::resources::{mesh::Vertex, texture::Texture};

use super::{mesh::Mesh, texture::Format};

/// Used to load default textures, textures from files or user defined textures.
pub struct TextureLoader {
    /// Sample the texture at different intervals. Bigger values will give worse quality textures.
    sampling: u32,
}
impl TextureLoader {
    /// Creates a default TextureLoader with sampling of 1. This sampling level does not alter the
    /// texture quality.
    pub fn new() -> Self {
        TextureLoader { sampling: 1 }
    }
    /// Used to generate a default texture.
    ///
    /// Allows the user to specify a default texture and build it through this function.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The default pattern to use to create the texture.
    ///
    /// # Example
    ///
    /// ```
    /// let tex_loader = TextureLoader::new(1);
    /// let texture = tex_loader.load_default_pattern(DefaultPattern::Checkered(100));
    /// ```
    pub fn load_default_texture(&self, texture: DefaultTexture) -> Texture {
        match texture {
            DefaultTexture::Checkered {
                width,
                height,
                nb_squares_width,
            } => {
                let format_nb_channels: usize = 3;
                let mut pixels =
                    Vec::<u8>::with_capacity(width as usize * height as usize * format_nb_channels);
                let mut square_length = width / nb_squares_width;
                if (width % nb_squares_width) != 0 {
                    square_length += 1;
                }
                // Setup the pixel values for the texture.
                for row in 0..height {
                    for col in 0..width {
                        let x_left = col % (2 * square_length) < square_length;
                        let y_lower = row % (2 * square_length) < square_length;
                        // Check in which quadrant the pixel is being drawn. Decide on color based
                        // on that.
                        let color = if !(x_left ^ y_lower) { 255 } else { 0 };
                        // Set the color of each channel to 0 or 255 (black or white).
                        for _ in 0..format_nb_channels {
                            // pixels[(row * size + col) as usize + channel] = color;
                            pixels.push(color);
                        }
                    }
                }
                // Finally, create the texture.
                Texture::from_pixels(width, height, &pixels, Format::RGB24).unwrap_or_else(|e| {
                    eprintln!("Could not create texture: {e}");
                    Texture::new(width, height, Format::RGB24)
                })
            }
        }
    }

    /// Loads a texture from a file.
    ///
    /// Given a file name which represents an image, load the image as a texture. Textures are
    /// stored in the `assets` folder in the root.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The name of the file that contains the texture.
    ///
    /// # Return
    ///
    /// The loaded texture if succesful, or an io error when the file failed to open or give a
    /// valid texture.
    pub fn load_texture_from_file(&self, file_name: &str) -> Result<Texture, std::io::Error> {
        println!("Loadgin texture from file: {}", file_name);
        todo!("Implement texture loading from file");
    }
    /// Getter for the sampling of the loader.
    pub fn sampling(&self) -> u32 {
        self.sampling
    }
}
/// A list of default textures that can be used to quickly get a texture.
pub enum DefaultTexture {
    /// A black and white checkered texture.
    Checkered {
        /// The size (in pixels) of the texture's width.
        width: u32,
        /// The size (in pixels) of the texture's height.
        height: u32,
        /// Number of squares that appear along the width of the texture.
        /// This will dictate their size and thus the number of square along the height.
        nb_squares_width: u32,
    },
}

/// Used to load default meshes, meshes from files or user defined meshes.
pub struct MeshLoader {
    /// Scale the position of the vertices by this amount on top of whatever scaling value is given
    /// during the mesh creation.
    scale: f32,
}
impl MeshLoader {
    /// Creates a default MeshLoader with base scaling of 1.
    pub fn new() -> Self {
        MeshLoader { scale: 1.0 }
    }
    /// Loads a default mesh.
    pub fn load_default_mesh(&self, mesh: DefaultMesh, texture_id: Option<u32>) -> Mesh {
        match mesh {
            DefaultMesh::Cube(size) => {
                let half_size = size / 2.0;
                let mut vertices = Vec::<Vertex>::with_capacity(8);
                // List of possible corner positions.
                let corners = [
                    DVec3::new(0.0, 0.0, 0.0) - half_size,
                    DVec3::new(size, 0.0, 0.0) - half_size,
                    DVec3::new(size, 0.0, size) - half_size,
                    DVec3::new(0.0, 0.0, size) - half_size,
                    DVec3::new(0.0, size, 0.0) - half_size,
                    DVec3::new(size, size, 0.0) - half_size,
                    DVec3::new(size, size, size) - half_size,
                    DVec3::new(0.0, size, size) - half_size,
                ];
                // List of possible uv coordinates.
                let uvs = [
                    DVec2::new(0.0, 0.0),
                    DVec2::new(1.0, 0.0),
                    DVec2::new(0.0, 1.0),
                    DVec2::new(1.0, 1.0),
                ];
                // Contains the indices of the triangle making up the mesh.
                let mut triangles = Vec::<u32>::with_capacity(24);
                // Fill in the vertices for each side.
                // -X
                vertices.push(Vertex::new(corners[4], uvs[0])); //0
                vertices.push(Vertex::new(corners[0], uvs[1])); //1
                vertices.push(Vertex::new(corners[3], uvs[3])); //2
                vertices.push(Vertex::new(corners[7], uvs[2])); //3
                                                                // +X
                vertices.push(Vertex::new(corners[1], uvs[0])); //4
                vertices.push(Vertex::new(corners[5], uvs[1])); //5
                vertices.push(Vertex::new(corners[6], uvs[3])); //6
                vertices.push(Vertex::new(corners[2], uvs[2])); //7
                                                                // -Y
                vertices.push(Vertex::new(corners[0], uvs[0])); //8
                vertices.push(Vertex::new(corners[1], uvs[1])); //9
                vertices.push(Vertex::new(corners[2], uvs[3])); //10
                vertices.push(Vertex::new(corners[3], uvs[2])); //11
                                                                // +Y
                vertices.push(Vertex::new(corners[5], uvs[0])); //12
                vertices.push(Vertex::new(corners[4], uvs[1])); //13
                vertices.push(Vertex::new(corners[7], uvs[3])); //14
                vertices.push(Vertex::new(corners[6], uvs[2])); //15
                                                                // -Z
                vertices.push(Vertex::new(corners[4], uvs[0])); //16
                vertices.push(Vertex::new(corners[5], uvs[1])); //17
                vertices.push(Vertex::new(corners[1], uvs[3])); //18
                vertices.push(Vertex::new(corners[0], uvs[2])); //19
                                                                // +Z//0
                vertices.push(Vertex::new(corners[3], uvs[0])); //20
                vertices.push(Vertex::new(corners[2], uvs[1])); //21
                vertices.push(Vertex::new(corners[6], uvs[3])); //22
                vertices.push(Vertex::new(corners[7], uvs[2])); //23

                // Fill up triangles indices
                for i in 0..6 {
                    // First triangle of each face.
                    triangles.push(4 * i + 0);
                    triangles.push(4 * i + 1);
                    triangles.push(4 * i + 2);
                    // Second triangle of each face.
                    triangles.push(4 * i + 2);
                    triangles.push(4 * i + 3);
                    triangles.push(4 * i + 0);
                }
                Mesh::new(texture_id, vertices, triangles)
            }
            DefaultMesh::SingleFace(size) => {
                let half_size = size / 2.0;
                let mut vertices = Vec::<Vertex>::with_capacity(4);
                // List of possible corner positions.
                let corners = [
                    DVec3::new(0.0, 0.0, 0.0) - half_size,
                    DVec3::new(size, 0.0, 0.0) - half_size,
                    DVec3::new(size, 0.0, size) - half_size,
                    DVec3::new(0.0, 0.0, size) - half_size,
                    DVec3::new(0.0, size, 0.0) - half_size,
                    DVec3::new(size, size, 0.0) - half_size,
                    DVec3::new(size, size, size) - half_size,
                    DVec3::new(0.0, size, size) - half_size,
                ];
                // List of possible uv coordinates.
                let uvs = [
                    DVec2::new(0.0, 0.0),
                    DVec2::new(1.0, 0.0),
                    DVec2::new(0.0, 1.0),
                    DVec2::new(1.0, 1.0),
                ];
                // Contains the indices of the triangle making up the mesh.
                let mut triangles = Vec::<u32>::with_capacity(24);
                // Fill in the vertices for each side.
                // +Z//0
                vertices.push(Vertex::new(corners[3], uvs[0])); //0
                vertices.push(Vertex::new(corners[2], uvs[1])); //1
                vertices.push(Vertex::new(corners[6], uvs[3])); //2
                vertices.push(Vertex::new(corners[7], uvs[2])); //3

                // Fill up triangles indices
                for i in 0..1 {
                    // First triangle of each face.
                    triangles.push(4 * i + 0);
                    triangles.push(4 * i + 1);
                    triangles.push(4 * i + 2);
                    // Second triangle of each face.
                    triangles.push(4 * i + 2);
                    triangles.push(4 * i + 3);
                    triangles.push(4 * i + 0);
                }
                Mesh::new(texture_id, vertices, triangles)
            }
        }
    }
    /// Loads a mesh from a file.
    ///
    /// Given a file name which holds an object representation, load it as a mesh. Objects are
    /// stored in the `assets` folder in the root.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The name of the file that contains the object.
    ///
    /// # Return
    ///
    /// The loaded mesh if succesful, or an io error when the file failed to open or give a
    /// valid object.
    pub fn load_mesh_from_file(&self, file_name: &str) -> Result<Mesh, std::io::Error> {
        println!("Loading mesh from file: {}", file_name);
        todo!("Implement mesh loading from file");
    }
    /// Getter for the scale of the loader.
    pub fn scale(&self) -> f32 {
        self.scale
    }
}
/// A list of default patterns that can be used to quickly get a texture.
pub enum DefaultMesh {
    /// A default cube with 8 vertices, one at each apex.
    ///
    /// - `f64` The size (in meters) of the cube's sides.
    Cube(f64),
    /// A square face with 4 vertices, one at each apex.
    /// Taken from the top (Z positive) face of the cube.
    ///
    /// - `f64` The size (in meters) of the face's sides.
    SingleFace(f64),
}
