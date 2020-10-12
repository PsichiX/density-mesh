pub mod coord;
pub mod generator;
pub mod live;
pub mod map;
pub mod mesh;
pub mod triangle;
mod utils;

use crate::{
    coord::Coord,
    generator::DensityMeshGenerator,
    map::DensityMap,
    mesh::{settings::GenerateDensityMeshSettings, DensityMesh, GenerateDensityMeshError},
};

/// Scalar type.
pub type Scalar = f32;

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
/// #![allow(deprecated)]
/// use density_mesh_core::prelude::*;
///
/// let map = DensityMap::new(2, 2, 1, vec![1, 2, 3, 1]).unwrap();
/// let settings = GenerateDensityMeshSettings {
///     points_separation: 0.5.into(),
///     visibility_threshold: 0.0,
///     steepness_threshold: 0.0,
///     ..Default::default()
/// };
/// assert_eq!(
///     generate_densitymesh_from_points_cloud(vec![], map, settings),
///     Ok(DensityMesh {
///         points: vec![
///             Coord { x: 0.0, y: 1.0 },
///             Coord { x: 0.0, y: 0.0 },
///             Coord { x: 1.0, y: 1.0 },
///             Coord { x: 1.0, y: 0.0 },
///         ],
///         triangles: vec![
///             Triangle { a: 0, b: 2, c: 1 },
///             Triangle { a: 2, b: 3, c: 1 },
///         ],
///     }),
/// );
/// ```
#[deprecated(
    since = "1.3.0",
    note = "Please use DensityMeshGenerator::process_wait() instead"
)]
pub fn generate_densitymesh_from_points_cloud(
    points: Vec<Coord>,
    map: DensityMap,
    settings: GenerateDensityMeshSettings,
) -> Result<DensityMesh, GenerateDensityMeshError> {
    DensityMeshGenerator::new(points, map, settings).process_wait()
}

/// Generate density mesh from points cloud, with callback that gets called on progress update.
///
/// # Arguments
/// * `points` - List of initial points.
/// * `map` - Density map.
/// * `settings` - Density mesh generation settings.
/// * `f` - Callback with progress arguments: `(current, limit, percentage)`.
///
/// # Returns
/// Density mesh or error.
///
/// # Examples
/// ```
/// #![allow(deprecated)]
/// use density_mesh_core::prelude::*;
///
/// let map = DensityMap::new(2, 2, 1, vec![1, 2, 3, 1]).unwrap();
/// let settings = GenerateDensityMeshSettings {
///     points_separation: 0.5.into(),
///     visibility_threshold: 0.0,
///     steepness_threshold: 0.0,
///     ..Default::default()
/// };
/// assert_eq!(
///     generate_densitymesh_from_points_cloud_tracked(
///         vec![],
///         map,
///         settings,
///         |c, l, p| println!("Progress: {}% ({} / {})", p * 100.0, c, l),
///     ),
///     Ok(DensityMesh {
///         points: vec![
///             Coord { x: 0.0, y: 1.0 },
///             Coord { x: 0.0, y: 0.0 },
///             Coord { x: 1.0, y: 1.0 },
///             Coord { x: 1.0, y: 0.0 },
///         ],
///         triangles: vec![
///             Triangle { a: 0, b: 2, c: 1 },
///             Triangle { a: 2, b: 3, c: 1 },
///         ],
///     }),
/// );
/// ```
#[deprecated(
    since = "1.3.0",
    note = "Please use DensityMeshGenerator::process_wait_tracked() instead"
)]
pub fn generate_densitymesh_from_points_cloud_tracked<F>(
    points: Vec<Coord>,
    map: DensityMap,
    settings: GenerateDensityMeshSettings,
    f: F,
) -> Result<DensityMesh, GenerateDensityMeshError>
where
    F: FnMut(usize, usize, Scalar),
{
    DensityMeshGenerator::new(points, map, settings).process_wait_tracked(f)
}

pub mod prelude {
    #[allow(deprecated)]
    pub use crate::{
        coord::*, generate_densitymesh_from_points_cloud,
        generate_densitymesh_from_points_cloud_tracked, generator::*, live::*, map::*,
        mesh::settings::*, mesh::*, triangle::*, Scalar,
    };
}
