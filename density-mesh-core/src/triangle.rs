use serde::{Deserialize, Serialize};

/// Triangle.
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Triangle {
    /// First point.
    pub a: usize,
    /// Second point.
    pub b: usize,
    /// Third point.
    pub c: usize,
}

impl From<[usize; 3]> for Triangle {
    fn from([a, b, c]: [usize; 3]) -> Self {
        Self { a, b, c }
    }
}
