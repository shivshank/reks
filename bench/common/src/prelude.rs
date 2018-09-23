pub use cgmath::{Vector3, vec3};
pub use components::*;
pub use systems::*;

#[derive(Debug, Default, Copy, Clone)]
pub struct DeltaTime(pub f64);
