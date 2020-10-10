use crate::Scalar;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Point coordinate.
///
/// # Examples
/// ```
/// use density_mesh_core::prelude::*;
///
/// let a = Coord::new(0.0, 0.0);
/// let b = Coord::new(2.0, 0.0);
/// assert_eq!((b - a).magnitude(), 2.0);
/// assert_eq!((b - a).sqr_magnitude(), 4.0);
/// assert_eq!((b - a).normalized(), Coord::new(1.0, 0.0));
/// assert_eq!((b - a).normalized().right(), Coord::new(0.0, -1.0));
/// assert_eq!(Coord::new(1.0, 0.0).dot(Coord::new(-1.0, 0.0)), -1.0);
/// ```
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coord {
    /// X value.
    pub x: Scalar,
    /// Y value.
    pub y: Scalar,
}

impl Coord {
    /// Create new point coordinate.
    ///
    /// # Arguments
    /// * `x` - X value.
    /// * `y` - Y value.
    #[inline]
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    /// Return squared length of the vector.
    #[inline]
    pub fn sqr_magnitude(self) -> Scalar {
        self.x * self.x + self.y * self.y
    }

    /// Return length of the vector.
    #[inline]
    pub fn magnitude(self) -> Scalar {
        self.sqr_magnitude().sqrt()
    }

    /// Return normalized vector (length equals to 1).
    #[inline]
    pub fn normalized(self) -> Self {
        self / self.magnitude()
    }

    /// Returns dot product (cosinus of the angle between two vectors when both are normalized).
    ///
    /// ```plain
    ///        self 1 other
    ///             ^
    ///             |
    /// other 0 <---*---> 0 other
    ///             |
    ///             v
    ///            -1
    ///           other
    /// ```
    /// # Arguments
    /// * `other` - Other vector.
    #[inline]
    pub fn dot(self, other: Self) -> Scalar {
        self.x * other.x + self.y * other.y
    }

    /// Return right vector.
    ///
    /// ```plain
    ///      ^
    /// self |
    ///      *---> right
    /// ```
    #[inline]
    pub fn right(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }
}

impl Add for Coord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<Scalar> for Coord {
    type Output = Self;

    fn add(self, other: Scalar) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl Sub for Coord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<Scalar> for Coord {
    type Output = Self;

    fn sub(self, other: Scalar) -> Self {
        Self {
            x: self.x - other,
            y: self.y - other,
        }
    }
}

impl Mul for Coord {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl Mul<Scalar> for Coord {
    type Output = Self;

    fn mul(self, other: Scalar) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div for Coord {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl Div<Scalar> for Coord {
    type Output = Self;

    fn div(self, other: Scalar) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Neg for Coord {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
