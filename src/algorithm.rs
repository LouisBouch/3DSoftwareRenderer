//! Implementation of different algorithms required by the renderer.

use glam::{DVec2, DVec4};

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
    Some((p_0 - l_0).dot(n) / denomi)
}

/// Given a triangle in 2D space, obtain the gradients of the barycentric coordinates.
///
/// Barycentric coordinates change linearly across the screen. This fact allows for faster
/// computation of the barycentric coordinates when it is done for multiple points. Incidentally,
/// these gradients can also be used to obtain the barycentric coordinates at the origin, allowing
/// us to compute the barycentric coordinates of any point quite easily.
///
/// Given u_alpha, u_beta, and u_gamma as the barycentric coordinate gradients of the triangle
/// a,b, and c, the coordinates of any point "p" can be found using:
/// alpha=-u_alpha.dot(c) + u_alpha.dot(p)
/// beta=-u_beta.dot(a) + u_beta.dot(p)
/// gamma=-u_gamma.dot(b) + u_gamma.dot(p)
///
/// # Arguments (Triangle vertices defined in CCW order)
///
/// * `a` - First vertex of triangle.
/// * `b` - Second vertex of triangle.
/// * `c` - Third vertex of triangle.
///
/// # Return
///
/// Returns the three gradients for the barycentric coordinates in the following order:
/// (u_alpha, u_beta, u_gamma).
#[inline(always)]
pub fn barycentric_gradients2(a: DVec2, b: DVec2, c: DVec2) -> (DVec2, DVec2, DVec2) {
    // The three edges of the triangle in CCW order.
    let ca = a - c;
    let bc = c - b;
    let ab = b - a;

    // The three "heights" of the triangle.
    let x = ca - (ca.dot(bc) / bc.dot(bc)) * bc; // Vector that goes from the triangle base bc to the vertex a.
    let y = ab - (ab.dot(ca) / ca.dot(ca)) * ca; // Vector that goes from the triangle base ca to the vertex b.
    let z = bc - (bc.dot(ab) / ab.dot(ab)) * ab; // Vector that goes from the triangle base ab to the vertex c.

    // The gradients.
    let u_alpha = x / x.dot(ca);
    let u_beta = y / y.dot(ab);
    let u_gamma = z / z.dot(bc);

    (u_alpha, u_beta, u_gamma)
}
/// Converts four 8bit numbers into a single u32. (Use u32::from_be_bytes instead)
///
/// # Arguments
///
/// * `b1` - The first 8 bits (starting from the left).
/// * `b2` - The next 8 bits.
/// * `b3` - The next next 8 bits.
/// * `b4` - The last 8 bits.
///
/// # Return
///
/// The u32 made from the four 8bit numbers (b1b2b3b4).
#[inline(always)]
pub fn u8s_to_u32(b1: u8, b2: u8, b3: u8, b4: u8) -> u32 {
    (b1 as u32) << 24 | (b2 as u32) << 16 | (b3 as u32) << 8 | (b4 as u32)
}
/// Given the vertices of a triangle, obtain its axis aligned bounding square.
///
/// # Return
///
/// The maximum and minimum values of x and y of the bounding square
/// in the following format:
/// (min_x, max_x, min_y, max_y)
#[inline(always)]
pub fn triangle_aabs(a: DVec2, b: DVec2, c: DVec2) -> (f64, f64, f64, f64) {
    let mut min_x = a.x.min(b.x).min(c.x).floor();
    let mut max_x = a.x.max(b.x).max(c.x).ceil();
    let mut min_y = a.y.min(b.y).min(c.y).floor();
    let mut max_y = a.y.max(b.y).max(c.y).ceil();
    min_x = if min_x < 0.0 { 0.0 } else { min_x };
    max_x = if max_x < 0.0 { 0.0 } else { max_x };
    min_y = if min_y < 0.0 { 0.0 } else { min_y };
    max_y = if max_y < 0.0 { 0.0 } else { max_y };
    (
        min_x,
        max_x,
        min_y,
        max_y,
    )
}
