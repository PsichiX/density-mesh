use crate::Scalar;
use serde::{Deserialize, Serialize};
use std::{num::ParseFloatError, ops::Range, str::FromStr};

/// Point separation source.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PointsSeparation {
    /// Each point has constant point separation.
    Constant(Scalar),
    /// Each point has local point separation that depends on the steepness value.
    /// When steepness is in range from 0 to 1, then steepness 0 maps to max and 1 maps to min.
    /// `(min, max)`
    SteepnessMapping(Scalar, Scalar),
}

impl PointsSeparation {
    /// Returns maximum of possible values.
    pub fn maximum(&self) -> Scalar {
        match self {
            Self::Constant(v) => *v,
            Self::SteepnessMapping(_, v) => *v,
        }
    }
}

impl From<Scalar> for PointsSeparation {
    fn from(value: Scalar) -> Self {
        Self::Constant(value)
    }
}

impl From<(Scalar, Scalar)> for PointsSeparation {
    fn from(value: (Scalar, Scalar)) -> Self {
        Self::SteepnessMapping(value.0, value.1)
    }
}

impl From<[Scalar; 2]> for PointsSeparation {
    fn from(value: [Scalar; 2]) -> Self {
        Self::SteepnessMapping(value[0], value[1])
    }
}

impl From<Range<Scalar>> for PointsSeparation {
    fn from(value: Range<Scalar>) -> Self {
        Self::SteepnessMapping(value.start, value.end)
    }
}

impl FromStr for PointsSeparation {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(found) = s.find("..") {
            let f = &s[..found];
            let t = &s[(found + 2)..];
            Ok(Self::SteepnessMapping(
                f.parse::<Scalar>()?,
                t.parse::<Scalar>()?,
            ))
        } else {
            Ok(Self::Constant(s.parse::<Scalar>()?))
        }
    }
}

impl ToString for PointsSeparation {
    fn to_string(&self) -> String {
        match self {
            Self::Constant(v) => v.to_string(),
            Self::SteepnessMapping(f, t) => format!("{}..{}", f, t),
        }
    }
}

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
    #[serde(default)]
    pub is_chunk: bool,
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
            is_chunk: false,
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
