pub mod coord;
pub mod generator;
pub mod map;
pub mod mesh;
pub mod triangle;

/// Scalar type.
pub type Scalar = f32;

pub mod prelude {
    pub use crate::{
        coord::*, generator::process_status::*, generator::*, map::*, mesh::points_separation::*,
        mesh::settings::*, mesh::*, triangle::*, Scalar,
    };
}
