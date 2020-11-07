use crate::{
    coord::Coord, mesh::settings::GenerateDensityMeshSettings, triangle::Triangle, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum ProcessingChange {
    FindingPoints {
        settings: GenerateDensityMeshSettings,
        tries: usize,
        /// [(coordinate, value, steepness, local point separation squared)]
        remaining: Vec<(Coord, Scalar, Scalar, Scalar)>,
        points: Vec<Coord>,
        progress_current: usize,
        progress_limit: usize,
    },
    Triangulate {
        settings: GenerateDensityMeshSettings,
        points: Vec<Coord>,
        progress_limit: usize,
    },
    RemoveInvisibleTriangles {
        settings: GenerateDensityMeshSettings,
        points: Vec<Coord>,
        triangles: Vec<Triangle>,
        progress_limit: usize,
    },
    Extrude {
        points: Vec<Coord>,
        triangles: Vec<Triangle>,
        size: Scalar,
        progress_limit: usize,
    },
}
