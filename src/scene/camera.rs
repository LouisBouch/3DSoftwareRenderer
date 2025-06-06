//! Contains everything realted to the [`Camera`].

use std::f64;

use glam::{DMat4, DQuat, DVec3, DVec4};

/// Contains the necessary information to define a [`Camera`].
///
/// ## The [`Camera`] implicitly uses the following for orientation (in camera space).
/// - Forward: **-[`DVec3`]::Z**
/// - Right: **[`DVec3`]::X**
/// - Up: **[`DVec3`]::Y**
pub struct Camera {
    /// Position of the camera in world space.
    position: DVec3,
    /// Quaternion defining the orientation of the [`Camera`].
    quat: DQuat,
    /// Transform that moves and rotates the [`Camera`] from the origin of the world to its position.
    /// Made from the position and quaternion.
    transform: DMat4,
    /// The type of projection the camera will use.
    projection: Projection,
    /// Velocity of the camera (in meters/sec).
    velocity: f64,
    /// Dictates how the camera will behave when moved with the mouse.
    camera_style: CameraStyle,
}
impl Default for Camera {
    /// Creates a default [`Camera`].
    ///
    /// # Default values
    ///
    /// - `position`: **[`DVec3`]::ZERO**
    /// - `quat`: **[`DQuat`]::default()**
    /// - `transform`: **[`DMat4`]::default()**
    /// - `near_clip`: **1.0**
    /// - `far_clip`: **10.0**
    /// - `aspect_ratio`: **16.0/9.0**
    /// - `hfov`: **90.0**
    fn default() -> Self {
        Camera::new_perspective(
            &DVec3::ZERO,
            &DQuat::default(),
            10.0,
            1000.0,
            16.0 / 9.0,
            90.0,
            CameraStyle::FPSLike,
        )
    }
}
/// Contains the available projection types for the [`Camera`].
pub enum Projection {
    /// Similar to what we see in our day to day life.
    Perspective {
        /// Distance to the near clipping plane of the view frustum.
        near_clip: f32,
        /// Distance to the far clipping plane of the view frustum.
        far_clip: f32,
        /// Width divided by height of the view frustum.
        aspect_ratio: f32,
        /// Horizontal fov of the view frustum (In degrees).
        hfov: f32,
    },
    /// This type of projection is depth invariant. Obejcts farther away do not seem smaller.
    Orthographic {
        /// Width of the orthographic projection (in meters).
        width: f32,
        /// Height of the orthographics projection (in meters).
        height: f32,
    },
}
/// Directions relative to the camera.
pub enum Direction {
    /// Forwards direction.
    Forwards,
    /// Backwards direction.
    Backwards,
    /// Left direction.
    Left,
    /// Right direction.
    Right,
    /// Up direction.
    Up,
    /// Down direction.
    Down,
}
/// Dictates how the camera's yaw/pitch/roll will be controlled.
pub enum CameraStyle {
    /// Yaw - rotates around the world's Y axis.
    /// Pitch - rotates aroudn the local X axis.
    /// Roll - no roll.
    FPSLike,
    /// Yaw - rotates around the local Y axis.
    /// Pitch - rotates aroudn the local X axis.
    /// Roll - rotates around the local Z axis.
    Free,
}
impl Camera {
    /// Creates a new [`Camera`] from its fields.
    pub fn new_perspective(
        position: &DVec3,
        quat: &DQuat,
        near_clip: f32,
        far_clip: f32,
        aspect_ratio: f32,
        hfov: f32,
        camera_style: CameraStyle,
    ) -> Self {
        let perspective = Projection::Perspective {
            near_clip,
            far_clip,
            aspect_ratio,
            hfov,
        };
        let mut c = Camera {
            position: position.clone(),
            quat: quat.clone(),
            transform: DMat4::IDENTITY,
            projection: perspective,
            velocity: 500.0,
            camera_style,
        };
        c.update_transform();
        c
    }
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
        self.update_transform();
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
        self.update_transform();
    }
    /// Rotates the camera by a quaternion.
    ///
    /// # Arguments
    ///
    /// * `rot` - The rotation to add to our current rotation.
    pub fn quat_rotation(&mut self, rot: &DQuat) {
        //q_total = q_second * q_first
        self.quat = rot.mul_quat(self.quat).normalize();
        // Ensure the transformation matrix stays up to date.
        self.update_transform();
    }
    /// Pitch the `camera` up or down (rotate around local X axis).
    ///
    /// # Arguments
    ///
    /// * `angle` - Angle by which the `camera` is pitched (in radians).
    /// `angle` > 0 means pitch up, and `angle` < 0 means pitch down (right hand rule).
    pub fn pitch(&mut self, angle: f64) {
        let axis = self.quat.mul_vec3(DVec3::X);
        let quat = DQuat::from_axis_angle(axis, angle);
        self.quat_rotation(&quat);
    }
    /// Yaw the `camera` right or left (rotate around local Y axis).
    ///
    /// # Arguments
    ///
    /// * `angle` - Angle by which the `camera` is yawed (in radians).
    /// `angle` > 0 means yaw left, and `angle` < 0 means yaw right (right hand rule).
    pub fn yaw(&mut self, angle: f64) {
        let axis = self.quat.mul_vec3(DVec3::Y);
        let quat = DQuat::from_axis_angle(axis, angle);
        self.quat_rotation(&quat);
    }
    /// Roll the `camera` CW or CCW (rotate around local Z axis).
    ///
    /// # Arguments
    ///
    /// * `angle` - Angle by which the `camera` is rolled (in radians).
    /// `angle` > 0 means CW rotation, and `angle` < 0 means CCW rotation (right hand rule).
    pub fn roll(&mut self, angle: f64) {
        let axis = self.quat.mul_vec3(DVec3::NEG_Z);
        let quat = DQuat::from_axis_angle(axis, angle);
        self.quat_rotation(&quat);
    }
    /// Sets the camera rotation with a quaternion.
    ///
    /// # Arguments
    ///
    /// * `rot` - The rotation to set our camera to.
    pub fn set_rotation(&mut self, rot: &DQuat) {
        //q_total = q_second * q_first
        self.quat = (*rot).normalize();
        // Ensure the transformation matrix stays up to date.
        self.update_transform();
    }
    /// Updates the entire transformation matrix to match the rotation and position of the camera.
    fn update_transform(&mut self) {
        self.transform = DMat4::from_translation(self.position) * DMat4::from_quat(self.quat);
    }
    /// Move the camera in the specified direction.
    ///
    /// # Arguments
    ///
    /// * `direction` - Direction in which to move the camera.
    /// * `distance` - Time over which the camera has moved (in seconds).
    pub fn move_cam(&mut self, dt: f64, direction: Direction) {
        let direction = self.quat.mul_vec3(match direction {
            Direction::Forwards => DVec3::NEG_Z,
            Direction::Backwards => DVec3::Z,
            Direction::Left => DVec3::NEG_X,
            Direction::Right => DVec3::X,
            Direction::Up => DVec3::Y,
            Direction::Down => DVec3::NEG_Y,
        });
        self.position += direction * dt * self.velocity;
        // Update transformation matrix to reflect the changes.
        self.update_transform();
    }
    /// Adds to the velocity of the camera.
    ///
    /// Positive values increase velocity, and negative value decrease it.
    pub fn add_velocity(&mut self, velocity: f64) {
        self.velocity = f64::max(0.0, self.velocity + velocity);
    }
    /// Yaw pitch and roll the camera according the the `camera_style` chosen.
    ///
    /// # Arguments
    ///
    /// * `yaw` - Pan the camera left/right (in radians).
    /// * `pitch` - Pitch the camera up/down (in radians).
    /// * `roll` - Roll the camera CW/CCW (in radians).
    pub fn yaw_pitch_roll(&mut self, yaw: f64, pitch: f64, roll: f64) {
        match self.camera_style {
            CameraStyle::FPSLike => {
                self.quat_rotation(&DQuat::from_axis_angle(DVec3::Y, yaw));
                self.pitch(pitch);
                // No roll in fps.

                // Correct pitch to ensure -90 <= pitch <= 90.
                let dot = f64::clamp(DVec3::Y.dot(self.quat.mul_vec3(DVec3::Y)), -1.0, 1.0);
                // Check if pitch is greater than 90 degrees.
                if dot < 0.0 {
                    let correction = dot.acos() - f64::consts::FRAC_PI_2;
                    // Ensure correction is in the right direction.
                    let sign = -DVec3::Y.dot(self.quat.mul_vec3(DVec3::NEG_Z)).signum();
                    // Now pitch correct.
                    self.pitch(correction * sign);
                }
            }
            CameraStyle::Free => {
                self.yaw(yaw);
                self.pitch(pitch);
                self.roll(roll);
            }
        }
    }
}
// Getters and setters.
impl Camera {
    /// Reference to the projection type of the [`Camera`].
    pub fn projection(&self) -> &Projection {
        &self.projection
    }
    /// Mutable reference to the projection type of the [`Camera`].
    pub fn projection_mut(&mut self) -> &mut Projection {
        &mut self.projection
    }
    /// Setter for the projection type of the [`Camera`].
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection
    }
    /// Gets the transformation matrix of the [`Camera`].
    pub fn transform(&self) -> &DMat4 {
        &self.transform
    }
    /// Gets the orientation of the [`Camera`].
    pub fn camera_orientation(&self) -> DVec3 {
        // Apply transformation to base orientation to get world orientation.
        let ori = self.transform.mul_vec4(-DVec4::Z);
        // Convert it to 3D.
        DVec3::new(ori.x, ori.y, ori.z)
    }
    /// Gets the velocity of the camera.
    pub fn velocity(&self) -> f64 {
        self.velocity
    }
    /// Sets the velocity of the camera.
    pub fn set_velocity(&mut self, velocity: f64) {
        self.velocity = velocity;
    }
    /// Reference to the quaternion defining the camera's rotation.
    pub fn quat(&self) -> &DQuat {
        &self.quat
    }
    /// Sets the camera movement style.
    pub fn set_camera_style(&mut self, camera_style: CameraStyle) {
        self.camera_style = camera_style;
    }
}
