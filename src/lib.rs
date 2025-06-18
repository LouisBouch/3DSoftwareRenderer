#![doc = include_str!("../README.md")]
#![warn(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc,
)]
pub mod app;
pub mod graphics;
pub mod inputs;
pub mod pipeline;
pub mod resources;
pub mod scene;
pub mod action;
pub mod algorithm;
