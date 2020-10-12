use crate::{
    coord::Coord,
    generator::DensityMeshGenerator,
    map::{DensityMap, DensityMapError},
    mesh::{settings::GenerateDensityMeshSettings, DensityMesh, GenerateDensityMeshError},
    triangle::Triangle,
    utils::{are_edges_connected, bake_final_mesh, does_triangle_share_edge},
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct BoundingBox {
    min: Coord,
    max: Coord,
}

impl BoundingBox {
    fn overlaps(&self, other: &Self) -> bool {
        self.max.x > other.min.x
            && self.max.y > other.min.y
            && self.min.x < other.max.x
            && self.min.y < other.max.y
    }
}

impl Into<(usize, usize, usize, usize)> for BoundingBox {
    fn into(self) -> (usize, usize, usize, usize) {
        let fx = self.min.x.max(0.0) as usize;
        let fy = self.min.y.max(0.0) as usize;
        let tx = self.max.x.max(0.0) as usize;
        let ty = self.max.y.max(0.0) as usize;
        (fx, fy, tx, ty)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RegionChange {
    bbox: BoundingBox,
    /// (first point index, second point index, plane origin, plane scaled normal)
    outline: Vec<(usize, usize, Coord, Coord)>,
    generator: DensityMeshGenerator,
}

/// Live density mesh processing status.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LiveProcessStatus {
    /// There are no pending changes.
    Idle,
    /// There are changes waiting for processing.
    InProgress,
    /// Inner mesh was updated during last processing.
    MeshChanged,
}

/// Density mesh with support for interactive region changes.
/// This struct allows you to make changes without splitting mesh into chunks.
/// Best for real-time applications.
///
/// # Examples
/// ```
/// use density_mesh_core::prelude::*;
///
/// let map = DensityMap::new(2, 4, 1, vec![255; 8]).unwrap();
/// let settings = GenerateDensityMeshSettings {
///     points_separation: 0.5.into(),
///     steepness_threshold: 0.0,
///     keep_invisible_triangles: true,
///     ..Default::default()
/// };
/// // create live density mesh.
/// let mut live = LiveDensityMesh::new(map, settings.clone());
/// // perform initial processing.
/// live.process_wait().unwrap();
/// // apply new data to inner density map region.
/// live.change_map(0, 2, 2, 2, vec![0; 4], settings);
/// // process changes.
/// live.process_wait().unwrap();
/// assert_eq!(live.map().values(), &[1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
/// assert_eq!(live.mesh().unwrap(), &DensityMesh {
///     points: vec![
///         Coord { x: 1.0, y: 0.0 },
///         Coord { x: 0.0, y: 0.0 },
///         Coord { x: 1.0, y: 2.0 },
///         Coord { x: 0.0, y: 2.0 },
///         Coord { x: 1.0, y: 1.0 },
///         Coord { x: 0.0, y: 1.0 },
///         Coord { x: 0.0, y: 3.0 },
///         Coord { x: 1.0, y: 3.0 },
///         Coord { x: 1.0, y: 2.0 },
///         Coord { x: 0.0, y: 2.0 },
///     ],
///     triangles: vec![
///         Triangle { a: 2, b: 4, c: 3 },
///         Triangle { a: 4, b: 5, c: 3 },
///         Triangle { a: 4, b: 0, c: 5 },
///         Triangle { a: 0, b: 1, c: 5 },
///         Triangle { a: 6, b: 7, c: 8 },
///         Triangle { a: 8, b: 9, c: 6 },
///     ],
/// });
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveDensityMesh {
    map: DensityMap,
    mesh: Option<DensityMesh>,
    queue: VecDeque<(BoundingBox, GenerateDensityMeshSettings)>,
    current: Option<RegionChange>,
}

impl LiveDensityMesh {
    /// Create new live density mesh instance.
    ///
    /// # Arguments
    /// * `map` - Density map.
    /// * `settings` - Density mesh generation settings.
    ///
    /// # Returns
    /// Live density mesh instance.
    pub fn new(map: DensityMap, settings: GenerateDensityMeshSettings) -> Self {
        let mut queue = VecDeque::with_capacity(1);
        queue.push_back((
            BoundingBox {
                min: Coord::new(0.0, 0.0),
                max: Coord::new(map.width() as Scalar, map.height() as Scalar),
            },
            settings,
        ));
        Self {
            map,
            mesh: None,
            queue,
            current: None,
        }
    }

    /// Get inner density map.
    pub fn map(&self) -> &DensityMap {
        &self.map
    }

    /// Get inner density mesh if one is already generated.
    pub fn mesh(&self) -> Option<&DensityMesh> {
        self.mesh.as_ref()
    }

    /// Tells if there are changes left to process.
    pub fn in_progress(&self) -> bool {
        self.current.is_some() || !self.queue.is_empty()
    }

    /// Process pending changes.
    ///
    /// # Arguments
    /// * `col` - Density map destination column.
    /// * `row` - Density map destination row.
    /// * `width` - Source data unscaled width.
    /// * `height` - Source data unscaled height.
    /// * `data` - Source data buffer.
    /// * `settings` - Density mesh generation settings applied for this change.
    ///
    /// # Returns
    /// Ok if successful or density map error.
    pub fn change_map(
        &mut self,
        col: usize,
        row: usize,
        width: usize,
        height: usize,
        data: Vec<u8>,
        mut settings: GenerateDensityMeshSettings,
    ) -> Result<(), DensityMapError> {
        self.map.change(col, row, width, height, data)?;
        let scale = self.map.scale() as Scalar;
        let extra = std::mem::replace(&mut settings.extrude_size, None).unwrap_or(0.0);
        let fx = col as Scalar * scale - extra;
        let fy = row as Scalar * scale - extra;
        let tx = fx + width as Scalar * scale + extra;
        let ty = fy + height as Scalar * scale + extra;
        self.queue.push_back((
            BoundingBox {
                min: Coord::new(fx, fy),
                max: Coord::new(tx, ty),
            },
            settings,
        ));
        Ok(())
    }

    /// Process all pending changes to internal mesh.
    ///
    /// # Returns
    /// Result with process status (Ok), or density mesh generation error.
    // TODO: split this code into sub-functions.
    pub fn process(&mut self) -> Result<LiveProcessStatus, GenerateDensityMeshError> {
        if self.current.is_none() && self.queue.is_empty() {
            return Ok(LiveProcessStatus::Idle);
        }
        if let Some(current) = std::mem::replace(&mut self.current, None) {
            let RegionChange {
                bbox,
                outline,
                generator,
            } = current;
            match generator.process()?.get_mesh_or_self() {
                Ok(mut new_mesh) => {
                    for p in &mut new_mesh.points {
                        p.x += bbox.min.x;
                        p.y += bbox.min.y;
                    }
                    if let Some(mesh) = std::mem::replace(&mut self.mesh, None) {
                        let DensityMesh { points, triangles } = new_mesh;
                        let triangles = triangles
                            .into_iter()
                            .filter(|t| {
                                let pa = points[t.a];
                                let pb = points[t.b];
                                let pc = points[t.c];
                                let c = (pa + pb + pc) / 3.0;
                                let mut samples = 0;
                                let mut count = 0;
                                for (a, b, o, n) in &outline {
                                    match does_triangle_share_edge(t.a, t.b, t.c, *a, *b) {
                                        0 => {}
                                        1 => {
                                            samples += 1;
                                            if (pa - *o).dot(*n) <= 0.0
                                                && (pb - *o).dot(*n) <= 0.0
                                                && (pc - *o).dot(*n) <= 0.0
                                            {
                                                count += 1;
                                            }
                                        }
                                        2 => {
                                            if (c - *o).dot(*n) <= 0.0 {
                                                return false;
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                samples == 0 || count < samples / 2
                            })
                            .collect::<Vec<_>>();
                        new_mesh = bake_final_mesh(points, triangles);
                        // TODO: fix duplicated points.
                        let count = mesh.points.len();
                        self.mesh = Some(DensityMesh {
                            points: mesh
                                .points
                                .into_iter()
                                .chain(new_mesh.points.into_iter())
                                .collect::<Vec<_>>(),
                            triangles: mesh
                                .triangles
                                .into_iter()
                                .chain(new_mesh.triangles.into_iter().map(|t| Triangle {
                                    a: t.a + count,
                                    b: t.b + count,
                                    c: t.c + count,
                                }))
                                .collect::<Vec<_>>(),
                        });
                    } else {
                        self.mesh = Some(new_mesh);
                    }
                    return Ok(LiveProcessStatus::MeshChanged);
                }
                Err(generator) => {
                    self.current = Some(RegionChange {
                        bbox,
                        outline,
                        generator,
                    });
                }
            }
        } else {
            if let Some((bbox, settings)) = self.queue.pop_front() {
                if let Some(mut mesh) = std::mem::replace(&mut self.mesh, None) {
                    // TODO: with capacity to reduce allocations.
                    let mut triangles = vec![];
                    mesh.triangles = mesh
                        .triangles
                        .iter()
                        .filter_map(|t| {
                            if Self::triangle_bbox(t, &mesh.points).overlaps(&bbox) {
                                triangles.push(*t);
                                None
                            } else {
                                Some(*t)
                            }
                        })
                        .collect::<Vec<_>>();
                    if triangles.is_empty() {
                        self.mesh = Some(mesh);
                    } else {
                        let edges = triangles
                            .iter()
                            .enumerate()
                            .flat_map(|(i, t)| vec![(i, t.a, t.b), (i, t.b, t.c), (i, t.c, t.a)])
                            .collect::<Vec<_>>();
                        let outline = edges
                            .iter()
                            .filter_map(|e1| {
                                if !edges.iter().any(|e2| {
                                    e1.0 != e2.0 && are_edges_connected(e1.1, e1.2, e2.1, e2.2)
                                }) {
                                    let o = mesh.points[e1.1];
                                    let n = (mesh.points[e1.2] - o).normalized().right();
                                    Some((e1.1, e1.2, o, n))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        let points_outer = outline
                            .iter()
                            .map(|(_, _, o, _)| *o - bbox.min)
                            .collect::<Vec<_>>();
                        let outline = outline
                            .iter()
                            .map(|(a, b, o, n)| {
                                (
                                    outline.iter().position(|(i, _, _, _)| a == i).unwrap(),
                                    outline.iter().position(|(i, _, _, _)| b == i).unwrap(),
                                    *o,
                                    *n,
                                )
                            })
                            .collect::<Vec<_>>();
                        let (fx, fy, tx, ty) = bbox.clone().into();
                        self.mesh = Some(bake_final_mesh(mesh.points, mesh.triangles));
                        self.current = Some(RegionChange {
                            bbox,
                            outline,
                            generator: DensityMeshGenerator::new(
                                points_outer,
                                self.map.crop(fx, fy, tx - fx, ty - fy),
                                settings,
                            ),
                        });
                    }
                } else {
                    let (fx, fy, tx, ty) = bbox.clone().into();
                    self.current = Some(RegionChange {
                        bbox,
                        outline: vec![],
                        generator: DensityMeshGenerator::new(
                            vec![],
                            self.map.crop(fx, fy, tx - fx, ty - fy),
                            settings,
                        ),
                    });
                }
            }
        }
        if self.current.is_some() || !self.queue.is_empty() {
            Ok(LiveProcessStatus::InProgress)
        } else {
            Ok(LiveProcessStatus::Idle)
        }
    }

    /// Process all pending changes to internal mesh until it's done.
    ///
    /// # Returns
    /// Ok when done, or density mesh generation error.
    pub fn process_wait(&mut self) -> Result<(), GenerateDensityMeshError> {
        loop {
            match self.process()? {
                LiveProcessStatus::Idle | LiveProcessStatus::MeshChanged => return Ok(()),
                _ => {}
            }
        }
    }

    fn triangle_bbox(triangle: &Triangle, points: &[Coord]) -> BoundingBox {
        let a = points[triangle.a];
        let b = points[triangle.b];
        let c = points[triangle.c];
        BoundingBox {
            min: Coord::new(a.x.min(b.x).min(c.x), a.y.min(b.y).min(c.y)),
            max: Coord::new(a.x.max(b.x).max(c.x), a.y.max(b.y).max(c.y)),
        }
    }
}
