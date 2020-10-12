use crate::{
    coord::Coord,
    map::DensityMap,
    mesh::{settings::GenerateDensityMeshSettings, DensityMesh, GenerateDensityMeshError},
    triangle::Triangle,
    Scalar,
};
use std::collections::HashMap;
use triangulation::{Delaunay, Point};

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

pub(crate) fn are_edges_connected(a_from: usize, a_to: usize, b_from: usize, b_to: usize) -> bool {
    (a_from == b_from && a_to == b_to) || (a_from == b_to && a_to == b_from)
}

pub(crate) fn does_triangle_share_edge(a: usize, b: usize, c: usize, from: usize, to: usize) -> u8 {
    let mut result = 0;
    if a == from || a == to {
        result += 1;
    }
    if b == from || b == to {
        result += 1;
    }
    if c == from || c == to {
        result += 1;
    }
    result
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

#[inline]
pub(crate) fn lerp(value: Scalar, from: Scalar, to: Scalar) -> Scalar {
    from + (to - from) * value.max(0.0).min(1.0)
}
