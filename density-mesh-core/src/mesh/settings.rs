use crate::{mesh::points_separation::PointsSeparation, Scalar};
use serde::{Deserialize, Serialize};

/// Settings of density mesh generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateDensityMeshSettings {
    /// Minimal points separation.
    #[serde(default = "GenerateDensityMeshSettings::default_points_separation")]
    pub points_separation: PointsSeparation,
    /// Minimal visibility treshold.
    #[serde(default = "GenerateDensityMeshSettings::default_visibility_threshold")]
    pub visibility_threshold: Scalar,
    /// Minimal steepness treshold.
    #[serde(default = "GenerateDensityMeshSettings::default_steepness_threshold")]
    pub steepness_threshold: Scalar,
    /// Limit of iterations when cannot find next available point.
    #[serde(default = "GenerateDensityMeshSettings::default_max_iterations")]
    pub max_iterations: usize,
    /// Optional extrude size.
    #[serde(default)]
    pub extrude_size: Option<Scalar>,
    /// Keep invisible triangles.
    #[serde(default)]
    pub keep_invisible_triangles: bool,
}

impl Default for GenerateDensityMeshSettings {
    fn default() -> Self {
        Self {
            points_separation: Self::default_points_separation(),
            visibility_threshold: Self::default_visibility_threshold(),
            steepness_threshold: Self::default_steepness_threshold(),
            max_iterations: Self::default_max_iterations(),
            extrude_size: None,
            keep_invisible_triangles: false,
        }
    }
}

impl GenerateDensityMeshSettings {
    fn default_points_separation() -> PointsSeparation {
        PointsSeparation::Constant(10.0)
    }

    fn default_visibility_threshold() -> Scalar {
        0.01
    }

    fn default_steepness_threshold() -> Scalar {
        0.01
    }

    fn default_max_iterations() -> usize {
        32
    }
}
