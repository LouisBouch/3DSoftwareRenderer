use glam::{DMat4, Vec2, Vec4};

struct Mesh {
    texture_id: Option<u32>,
    world_transfrom: DMat4,
    vertices: Vec<Vertex>,
    triangles: Vec<u32>,
}
struct Vertex {
    position: Vec4,
    uv: Vec2,
}
