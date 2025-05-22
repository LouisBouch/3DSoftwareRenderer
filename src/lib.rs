#![doc = include_str!("../README.md")]
//! # This is a bigger test.
//! oioioi
//! ## This is a test.
//! Oi
//!
//! - t1
//! - [ ] t2
//! - [x] t3
//!
//! ## Example
//! ```
//! let o = 2;
//! println!("oi {}", o);
//! ```
//!
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

// Default tests
#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {
        let result = 2+2;
        assert_eq!(result, 4);
    }
}
