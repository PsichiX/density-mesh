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
