use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Neg, Sub},
};
use triangulation::{Delaunay, Point};

/// Scalar type.
pub type Scalar = f32;

/// Point coordinate.
///
/// # Examples
/// ```
/// use density_mesh_core::*;
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

/// Error thrown during density map generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DensityMapError {
    /// Wrong data length.
    /// (provided, expected)
    WrongDataLength(usize, usize),
}

/// Density map that contains density data and steepness per pixel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DensityMap {
    width: usize,
    height: usize,
    scale: usize,
    data: Vec<Scalar>,
    steepness: Vec<Scalar>,
}

impl DensityMap {
    /// Create new density map.
    ///
    /// # Arguments
    /// * `width` - Columns.
    /// * `height` - Rows.
    /// * `scale` - Scale.
    /// * `data` - Raw pixel data.
    ///
    /// # Returns
    /// Density map or error.
    ///
    /// # Examples
    /// ```
    /// use density_mesh_core::*;
    ///
    /// assert!(DensityMap::new(2, 2, 1, vec![0, 1, 2, 3]).is_ok());
    /// assert_eq!(
    ///     DensityMap::new(1, 2, 1, vec![0, 1, 2, 3]),
    ///     Err(DensityMapError::WrongDataLength(4, 2)),
    /// );
    /// ```
    pub fn new(
        width: usize,
        height: usize,
        scale: usize,
        data: Vec<u8>,
    ) -> Result<Self, DensityMapError> {
        if data.len() == width * height {
            let data = data
                .into_iter()
                .map(|v| v as Scalar / 255.0)
                .collect::<Vec<_>>();
            let steepness = (0..data.len())
                .map(|i| {
                    let col = (i % width) as isize;
                    let row = (i / width) as isize;
                    let mut result = 0.0;
                    for x in (col - 1)..(col + 1) {
                        for y in (row - 1)..(row + 1) {
                            let a = Self::raw_value(x, y, width, height, &data);
                            let b = Self::raw_value(x + 1, y, width, height, &data);
                            let c = Self::raw_value(x + 1, y + 1, width, height, &data);
                            let d = Self::raw_value(x, y + 1, width, height, &data);
                            let ab = (a - b).abs();
                            let cd = (c - d).abs();
                            let ac = (a - c).abs();
                            let bd = (b - d).abs();
                            let ad = (a - d).abs();
                            let bc = (b - c).abs();
                            result += (ab + cd + ac + bd + ad + bc) / 12.0;
                        }
                    }
                    result
                })
                .collect::<Vec<_>>();
            Ok(Self {
                width,
                height,
                scale,
                data,
                steepness,
            })
        } else {
            Err(DensityMapError::WrongDataLength(data.len(), width * height))
        }
    }

    /// Returns scale.
    pub fn scale(&self) -> usize {
        self.scale
    }

    /// Returns scaled width.
    pub fn width(&self) -> usize {
        self.width * self.scale.max(1)
    }

    /// Returns scaled height.
    pub fn height(&self) -> usize {
        self.height * self.scale.max(1)
    }

    /// Returns unscaled width.
    pub fn unscaled_width(&self) -> usize {
        self.width
    }

    /// Returns unscaled height.
    pub fn unscaled_height(&self) -> usize {
        self.height
    }

    /// Returns values buffer.
    pub fn values(&self) -> &[Scalar] {
        &self.data
    }

    /// Returns steepness buffer.
    pub fn steepness(&self) -> &[Scalar] {
        &self.steepness
    }

    /// Returns value at given point or 0 if out of bounds.
    ///
    /// # Arguments
    /// * `point` - (X, Y)
    pub fn value_at_point(&self, point: (isize, isize)) -> Scalar {
        let scale = self.scale.max(1) as isize;
        let col = point.0 / scale;
        let row = point.1 / scale;
        if col >= 0 && col < self.width as _ && row >= 0 && row < self.height as _ {
            self.data
                .get(row as usize * self.width + col as usize)
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Returns steepness at given point or 0 if out of bounds.
    ///
    /// # Arguments
    /// * `point` - (X, Y)
    pub fn steepness_at_point(&self, point: (isize, isize)) -> Scalar {
        let scale = self.scale.max(1) as isize;
        let col = point.0 / scale;
        let row = point.1 / scale;
        if col >= 0 && col < self.width as _ && row >= 0 && row < self.height as _ {
            self.steepness
                .get(row as usize * self.width + col as usize)
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// Returns iterator over values and steepness buffers.
    ///
    /// # Examples
    /// ```
    /// use density_mesh_core::*;
    ///
    /// let map = DensityMap::new(2, 2, 1, vec![2, 2, 4, 4])
    ///     .unwrap()
    ///     .value_steepness_iter()
    ///     .collect::<Vec<_>>();
    /// assert_eq!(
    ///     map,
    ///     vec![
    ///         (0, 0, 0.007843138, 0.011764706),
    ///         (1, 0, 0.007843138, 0.011764707),
    ///         (0, 1, 0.015686275, 0.01633987),
    ///         (1, 1, 0.015686275, 0.01633987),
    ///     ],
    /// );
    /// ```
    pub fn value_steepness_iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (usize, usize, Scalar, Scalar)> + 'a {
        self.data
            .iter()
            .zip(self.steepness.iter())
            .enumerate()
            .map(move |(i, (v, s))| (i % self.width, i / self.width, *v, *s))
    }

    fn raw_value(x: isize, y: isize, w: usize, h: usize, data: &[Scalar]) -> Scalar {
        if x >= 0 && x < w as _ && y >= 0 && y < h as _ {
            data[y as usize * w + x as usize]
        } else {
            0.0
        }
    }
}

/// Settings of density mesh generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDensityMeshSettings {
    /// Minimal points separation.
    #[serde(default = "GenerateDensityMeshSettings::default_points_separation")]
    pub points_separation: Scalar,
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
}

impl Default for GenerateDensityMeshSettings {
    fn default() -> Self {
        Self {
            points_separation: Self::default_points_separation(),
            visibility_threshold: Self::default_visibility_threshold(),
            steepness_threshold: Self::default_steepness_threshold(),
            max_iterations: Self::default_max_iterations(),
            extrude_size: None,
        }
    }
}

impl GenerateDensityMeshSettings {
    fn default_points_separation() -> Scalar {
        10.0
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

/// Error thrown during density mesh generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenerateDensityMeshError {
    /// Density map error.
    DensityMap(DensityMapError),
    /// Failed points triangulation.
    FailedTriangulation,
}

/// Density mesh.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DensityMesh {
    /// List of points.
    pub points: Vec<Coord>,
    /// List of triangles.
    pub triangles: Vec<Triangle>,
}

/// Generate density mesh from points cloud.
///
/// # Arguments
/// * `points` - List of initial points.
/// * `map` - Density map.
/// * `settings` - Density mesh generation settings.
///
/// # Returns
/// Density mesh or error.
///
/// # Examples
/// ```
/// use density_mesh_core::*;
///
/// let map = DensityMap::new(2, 2, 1, vec![1, 2, 3, 1]).unwrap();
/// let settings = GenerateDensityMeshSettings {
///     points_separation: 0.5,
///     visibility_threshold: 0.0,
///     steepness_threshold: 0.0,
///     ..Default::default()
/// };
/// assert_eq!(
///     generate_densitymesh_from_points_cloud(vec![], map, &settings),
///     Ok(DensityMesh {
///         points: vec![
///             Coord { x: 0.0, y: 1.0 },
///             Coord { x: 0.0, y: 0.0 },
///             Coord { x: 1.0, y: 0.0 },
///             Coord { x: 1.0, y: 1.0 },
///         ],
///         triangles: vec![
///             Triangle { a: 0, b: 2, c: 1 },
///             Triangle { a: 0, b: 3, c: 2 },
///         ],
///     }),
/// );
/// ```
pub fn generate_densitymesh_from_points_cloud(
    mut points: Vec<Coord>,
    map: DensityMap,
    settings: &GenerateDensityMeshSettings,
) -> Result<DensityMesh, GenerateDensityMeshError> {
    let scale = map.scale().max(1);
    let mut remaining = map
        .value_steepness_iter()
        .map(|(x, y, v, s)| {
            let x = (x * scale) as Scalar;
            let y = (y * scale) as Scalar;
            (Coord::new(x, y), v, s)
        })
        .filter(|(_, v, s)| *v > settings.visibility_threshold && *s > settings.steepness_threshold)
        .collect::<Vec<_>>();
    points.reserve(remaining.len());
    let mds = settings.points_separation * settings.points_separation;
    let mut tries = settings.max_iterations.max(1);
    loop {
        if !points.is_empty() {
            remaining
                .retain(|(p1, _, _)| points.iter().all(|p2| (*p2 - *p1).sqr_magnitude() > mds));
        }
        if let Some((index, (point, _, _))) = remaining
            .iter()
            .enumerate()
            .max_by(|a, b| (a.1).2.partial_cmp(&(b.1).2).unwrap())
        {
            points.push(*point);
            remaining.swap_remove(index);
            tries = settings.max_iterations.max(1);
        } else {
            if tries > 0 {
                tries -= 1;
            } else {
                break;
            }
        }
        if remaining.is_empty() {
            break;
        }
    }
    let mut triangles = triangulate(&points)?
        .iter()
        .filter(|t| is_triangle_visible(points[t.a], points[t.b], points[t.c], &map, settings))
        .copied()
        .collect::<Vec<_>>();
    if let Some(size) = settings.extrude_size {
        let (p, t) = extrude(&points, &triangles, size);
        points.extend(p);
        triangles.extend(t);
        Ok(bake_final_mesh(points, triangles))
    } else {
        Ok(bake_final_mesh(points, triangles))
    }
}

fn triangulate(points: &[Coord]) -> Result<Vec<Triangle>, GenerateDensityMeshError> {
    let points = points
        .iter()
        .map(|p| Point::new(p.x, p.y))
        .collect::<Vec<_>>();
    if let Some(del) = Delaunay::new(&points) {
        Ok(del
            .dcel
            .vertices
            .chunks(3)
            .map(|t| [t[0], t[1], t[2]].into())
            .collect::<Vec<_>>())
    } else {
        Err(GenerateDensityMeshError::FailedTriangulation)
    }
}

fn is_triangle_visible(
    a: Coord,
    b: Coord,
    c: Coord,
    map: &DensityMap,
    settings: &GenerateDensityMeshSettings,
) -> bool {
    let fx = (a.x as isize).min(b.x as isize).min(c.x as isize);
    let fy = (a.y as isize).min(b.y as isize).min(c.y as isize);
    let tx = (a.x as isize).max(b.x as isize).max(c.x as isize);
    let ty = (a.y as isize).max(b.y as isize).max(c.y as isize);
    let nab = (b - a).right();
    let nbc = (c - b).right();
    let nca = (a - c).right();
    let mut count = 0;
    let mut samples = 0;
    for y in fy..=ty {
        for x in fx..=tx {
            let p = Coord::new(x as _, y as _);
            if (p - a).dot(nab) >= 0.0 && (p - b).dot(nbc) >= 0.0 && (p - c).dot(nca) >= 0.0 {
                samples += 1;
                if is_point_visible(p, map, settings) {
                    count += 1;
                }
            }
        }
    }
    count as Scalar / samples as Scalar > 0.5
}

fn is_point_visible(p: Coord, map: &DensityMap, settings: &GenerateDensityMeshSettings) -> bool {
    map.value_at_point((p.x as _, p.y as _)) > settings.visibility_threshold
}

fn extrude(points: &[Coord], triangles: &[Triangle], size: Scalar) -> (Vec<Coord>, Vec<Triangle>) {
    let edges = triangles
        .iter()
        .enumerate()
        .flat_map(|(i, t)| vec![(i, t.a, t.b), (i, t.b, t.c), (i, t.c, t.a)])
        .collect::<Vec<_>>();
    let outline = edges
        .iter()
        .filter(|e1| {
            !edges
                .iter()
                .any(|e2| e1.0 != e2.0 && are_edges_connected(e1.1, e1.2, e2.1, e2.2))
        })
        .collect::<Vec<_>>();
    let offsets = outline
        .iter()
        .map(|(_, m, n)| {
            let i = *m;
            let p = outline.iter().find(|(_, _, p)| p == m).unwrap().1;
            let p = points[p];
            let m = points[*m];
            let n = points[*n];
            let pm = -(m - p).normalized().right();
            let mn = -(n - m).normalized().right();
            (i, m + (pm + mn).normalized() * size)
        })
        .collect::<Vec<_>>();
    let triangles = outline
        .into_iter()
        .flat_map(|(_, a, b)| {
            let ea = offsets.iter().position(|(ea, _)| ea == a).unwrap() + points.len();
            let eb = offsets.iter().position(|(eb, _)| eb == b).unwrap() + points.len();
            vec![[*b, *a, ea].into(), [ea, eb, *b].into()]
        })
        .collect::<Vec<_>>();
    (
        offsets.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
        triangles,
    )
}

fn are_edges_connected(a_from: usize, a_to: usize, b_from: usize, b_to: usize) -> bool {
    (a_from == b_from && a_to == b_to) || (a_from == b_to && a_to == b_from)
}

fn bake_final_mesh(points: Vec<Coord>, mut triangles: Vec<Triangle>) -> DensityMesh {
    let mut mapping = HashMap::with_capacity(points.len());
    let mut new_points = Vec::with_capacity(points.len());
    for (i, p) in points.iter().enumerate() {
        if triangles.iter().any(|t| i == t.a || i == t.b || i == t.c) {
            new_points.push(*p);
            if !new_points.is_empty() {
                mapping.insert(i, new_points.len() - 1);
            }
        }
    }
    for t in &mut triangles {
        t.a = mapping[&t.a];
        t.b = mapping[&t.b];
        t.c = mapping[&t.c];
    }
    DensityMesh {
        points: new_points,
        triangles,
    }
}
