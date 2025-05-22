//! Contains everything realted to the camera.

use glam::{self, DMat3, DMat4, DQuat, DVec3};

/// Contains the necessary information to define a camera.
///
/// ## The camera implicitly uses the following for orientation
/// - Forward: **-[`DVec3`]::Z**
/// - Right: **[`DVec3`]::X**
/// - Up: **[`DVec3`]::Y**
pub struct Camera {
    /// Position of the camera in world space.
    position: DVec3,
    /// Quaternion defining the orientation of the camera.
    quat: DQuat,
    /// Transform that moves and rotates the camera from the origin of the world to its position.
    /// Made from the position and quaternion.
    transform: DMat4,
    /// Distance to the near clipping plane of the view frustum.
    near_clip: f32,
    /// Distance to the far clipping plane of the view frustum.
    far_clip: f32,
    /// Width divided by height of the view frustum.
    aspect_ratio: f32,
    /// Horizontal fov of the view frustum.
    hfov: f32,
}
impl Default for Camera {
    /// Creates a default camera.
    ///
    /// ### Default values
    /// - `position`: **[`DVec3`]::ZERO**
    /// - `quat`: **[`DQuat`]::default()**
    /// - `transform`: **[`DMat4`]::default()**
    /// - `near_clip`: **1.0**
    /// - `far_clip`: **10.0**
    /// - `aspect_ratio`: **16.0/9.0**
    /// - `hfov`: **90.0**
    fn default() -> Self {
        Camera {
            position: DVec3::ZERO,
            quat: DQuat::default(),
            transform: DMat4::default(),
            near_clip: 1.0,
            far_clip: 10.0,
            aspect_ratio: 16.0 / 9.0,
            hfov: 90.0,
        }
    }
}

impl Camera {
    /// Gets an immutable reference to the position vector.
    ///
    /// # Returns
    ///
    /// Reference to the position of the camera.
    pub fn position(&self) -> &DVec3 {
        &self.position
    }
    /// Adds a vector to the camera position.
    ///
    /// # Arguments
    ///
    /// * `change` - The vector to add to the current position.
    pub fn add_position(&mut self, change: &DVec3) {
        self.position += change;
        // Update transformation matrix to reflect the changes.
        self.update_transform_translation();
    }
    /// Sets the position of the camera.
    ///
    /// # Arguments
    ///
    /// * `new_pos` - The new position vector for the camera.
    pub fn set_position(&mut self, new_pos: &DVec3) {
        self.position[0] = new_pos[0];
        self.position[1] = new_pos[1];
        self.position[2] = new_pos[2];
        // Update transformation matrix to reflect the changes.
        self.update_transform_translation();
    }
    /// Rotates the camera by a quaternion.
    ///
    /// # Arguments
    ///
    /// * `rot` - The rotation to add to our current rotation.
    pub fn rotate(&mut self, rot: &DQuat) {
        //q_total = q_second * q_first
        self.quat = rot.mul_quat(self.quat).normalize();
        // Ensure the transformation matrix stays up to date.
        self.update_transform_rotation();
    }
    /// Updates the upper 3x3 section of the transformation matrix to match the rotation of the camera.
    fn update_transform_rotation(&mut self) {
        // Updates upper 3x3 matrix where the rotation part resides.
        let rot_matrix = DMat3::from_quat(self.quat);
        for col in 0..3 {
            let tran_col = self.transform.col_mut(col);
            let rot_col = rot_matrix.col(col);
            for row in 0..3 {
                tran_col[row] = rot_col[row];
            }
        }
    }
    /// Updates the right most column of the transformation matrix to match the position of the camera.
    fn update_transform_translation(&mut self) {
        let mut w_axis = self.transform.w_axis;
        let pos = self.position;
        w_axis[0] = pos[0];
        w_axis[1] = pos[1];
        w_axis[2] = pos[2];
    }
    /// Updates the entire transformation matrix to match the rotation and position of the camera.
    fn update_transform_all(&mut self) {
        self.update_transform_rotation();
        self.update_transform_translation();
    }
}
