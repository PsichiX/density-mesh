pub mod points_separation;
pub mod settings;

use crate::{coord::Coord, map::DensityMapError, triangle::Triangle};
use serde::{Deserialize, Serialize};

/// Error thrown during density mesh generation.
#[derive(Debug, Clone, PartialEq)]
pub enum GenerateDensityMeshError {
    /// Density map error.
    DensityMap(DensityMapError),
    /// Trying to process unitialized generator.
    UninitializedGenerator,
    /// Failed points triangulation.
    FailedTriangulation,
    /// There is no density mesh created.
    NothingCreated,
}

/// Density mesh.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DensityMesh {
    /// List of points.
    pub points: Vec<Coord>,
    /// List of triangles.
    pub triangles: Vec<Triangle>,
}
