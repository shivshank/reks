use prelude::*;

/// basic euler integration
pub fn integrate_velocity(pos: &mut Vector3<f32>, vel: &Vector3<f32>, dt: f64) {
    *pos += *vel * (dt as f32);
}
