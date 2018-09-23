pub extern crate cgmath;

// Unfortunately, because of the orhpan rules, we need to have specs impls in the common crate
// Specs is re-exported so that the specs tests are using the same specs version.
#[cfg(feature="specs_impls")]
pub extern crate specs;

pub mod prelude;
pub mod components;
pub mod systems;
