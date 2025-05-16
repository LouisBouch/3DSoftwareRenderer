//! This crate allows for the rendering of triangular meshes through a software
//! renderer. It uses fps-like controls to allow users to move within the
//! environment, see [`inputs`] for more information regarding movement.
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
#![warn(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc,
)]
pub mod app;
pub mod graphics;
pub mod inputs;
mod pipeline;
mod resources;
mod scene;

/// Adds two numbers.
///
/// Given `left` and `right`, return their sum.
///
/// # Arguments
///
/// * `left` - First number to add
/// * `right` - Second number to add
///
/// # Returns
///
/// The sum of `left` and `right`.
///
/// # Panics
///
/// This function does not panic.
///
/// # Examples
///
/// ```
/// let result = add(1,2);
/// assert_eq!(result, 3);
/// ```
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
