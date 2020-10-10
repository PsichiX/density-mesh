use crate::{
    coord::Coord,
    generator::DensityMeshGenerator,
    map::DensityMap,
    mesh::{settings::GenerateDensityMeshSettings, DensityMesh, GenerateDensityMeshError},
    triangle::Triangle,
    Scalar,
};
use std::collections::HashMap;
use triangulation::{Delaunay, Point};

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
/// use density_mesh_core::prelude::*;
///
/// let map = DensityMap::new(2, 2, 1, vec![1, 2, 3, 1]).unwrap();
/// let settings = GenerateDensityMeshSettings {
///     points_separation: 0.5,
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
pub fn generate_densitymesh_from_points_cloud(
    points: Vec<Coord>,
    map: DensityMap,
    settings: GenerateDensityMeshSettings,
) -> Result<DensityMesh, GenerateDensityMeshError> {
    let mut generator = DensityMeshGenerator::new(points, map, settings);
    loop {
        match generator.process()?.get_mesh_or_self() {
            Ok(mesh) => return Ok(mesh),
            Err(gen) => generator = gen,
        }
    }
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
/// use density_mesh_core::prelude::*;
///
/// let map = DensityMap::new(2, 2, 1, vec![1, 2, 3, 1]).unwrap();
/// let settings = GenerateDensityMeshSettings {
///     points_separation: 0.5,
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
pub fn generate_densitymesh_from_points_cloud_tracked<F>(
    points: Vec<Coord>,
    map: DensityMap,
    settings: GenerateDensityMeshSettings,
    mut f: F,
) -> Result<DensityMesh, GenerateDensityMeshError>
where
    F: FnMut(usize, usize, Scalar),
{
    let mut generator = DensityMeshGenerator::new(points, map, settings);
    let (c, l, p) = generator.progress();
    f(c, l, p);
    loop {
        let gen = generator.process()?;
        let (c, l, p) = gen.progress();
        f(c, l, p);
        match gen.get_mesh_or_self() {
            Ok(mesh) => return Ok(mesh),
            Err(gen) => generator = gen,
        }
    }
}

pub(crate) fn triangulate(points: &[Coord]) -> Result<Vec<Triangle>, GenerateDensityMeshError> {
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

pub(crate) fn is_triangle_visible(
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
                if is_point_visible((x, y), map, settings) {
                    count += 1;
                }
            }
        }
    }
    count as Scalar / samples as Scalar > 0.5
}

fn is_point_visible(
    pos: (isize, isize),
    map: &DensityMap,
    settings: &GenerateDensityMeshSettings,
) -> bool {
    map.value_at_point(pos) > settings.visibility_threshold
}

pub(crate) fn extrude(
    points: &[Coord],
    triangles: &[Triangle],
    size: Scalar,
) -> (Vec<Coord>, Vec<Triangle>) {
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

pub(crate) fn bake_final_mesh(points: Vec<Coord>, mut triangles: Vec<Triangle>) -> DensityMesh {
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
