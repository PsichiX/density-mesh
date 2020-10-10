pub mod coord;
pub mod generator;
pub mod map;
pub mod mesh;
pub mod triangle;
pub mod utils;

/// Scalar type.
pub type Scalar = f32;

pub mod prelude {
    pub use crate::coord::*;
    pub use crate::generator::*;
    pub use crate::map::*;
    pub use crate::mesh::settings::*;
    pub use crate::mesh::*;
    pub use crate::triangle::*;
    pub use crate::utils::*;
    pub use crate::Scalar;
}
