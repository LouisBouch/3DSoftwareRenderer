//! Contains everytihng that will be needed to render the scene.

use crate::resources::texture::TextureCatalog;
mod clipper;
mod culling;
mod geometry;
mod rasterizer;
mod transforms;

/// Contains cached values imprtant for rendering.
pub struct PipeLine {
    texture_catalog: TextureCatalog,
}
