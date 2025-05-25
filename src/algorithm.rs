//! Implementation of different algorithms required by the renderer.

use glam::DVec4;

/// Line plane intersection detection in 4D. Obtains the intersection position 
/// between them if it exists.
///
/// # Arguments
///
/// * `p_0` - A point in the plane.
/// * `n` - The normal to the plane.
/// * `l_0` - A point on the line.
/// * `l` - A vector pointing in the direction of the line.
///
/// # Return
///
/// Given the line formula p=l_0 + t*l, returns the corresponding t that will
/// result in the intersection position.
#[inline(always)]
pub fn lin_plane_intersect4(p_0: DVec4, n: DVec4, l_0: DVec4, l: DVec4) -> Option<f64> {
    // Direction vector from a to b.
    let denomi = l.dot(n);
    // Check for parallel line and normal
    if denomi.abs() < 1e-12 {
        return None;
    }
    Some((p_0 - l_0).dot(n)/denomi)
}
