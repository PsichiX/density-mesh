pub mod process_status;
mod processing_change;

use crate::{
    coord::Coord,
    generator::{process_status::ProcessStatus, processing_change::ProcessingChange},
    map::{DensityMap, DensityMapError},
    mesh::{
        points_separation::PointsSeparation, settings::GenerateDensityMeshSettings, DensityMesh,
        GenerateDensityMeshError,
    },
    triangle::Triangle,
    Scalar,
};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use triangulation::{Delaunay, Point};

#[cfg(feature = "parallel")]
macro_rules! into_iter {
    ($v:expr) => {
        $v.into_par_iter()
    };
}

#[cfg(not(feature = "parallel"))]
macro_rules! into_iter {
    ($v:expr) => {
        $v.into_iter()
    };
}

/// Generate density mesh with region changes.
/// For now it recalculates mesh from whole density map data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DensityMeshGenerator {
    map: DensityMap,
    mesh: Option<DensityMesh>,
    /// [([points], settings)]
    queue: VecDeque<(Vec<Coord>, GenerateDensityMeshSettings)>,
    current: Option<ProcessingChange>,
}

impl DensityMeshGenerator {
    /// Create new generator.
    ///
    /// # Arguments
    /// * `points` - Initial points.
    /// * `map` - Density map.
    /// * `settings` - Settings.
    ///
    /// # Returns
    /// New generator instance.
    pub fn new(points: Vec<Coord>, map: DensityMap, settings: GenerateDensityMeshSettings) -> Self {
        let mut queue = VecDeque::with_capacity(1);
        queue.push_back((points, settings));
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

    pub fn into_mesh(self) -> Option<DensityMesh> {
        self.mesh
    }

    /// Tells if there are changes left to process.
    pub fn in_progress(&self) -> bool {
        self.current.is_some() || !self.queue.is_empty()
    }

    /// Get processing progress.
    ///
    /// # Returns
    /// `(current, limit, percentage)`
    pub fn progress(&self) -> (usize, usize, Scalar) {
        match &self.current {
            Some(ProcessingChange::FindingPoints {
                progress_current,
                progress_limit,
                ..
            }) => (
                *progress_current,
                *progress_limit,
                *progress_current as Scalar / *progress_limit as Scalar,
            ),
            Some(ProcessingChange::Triangulate { progress_limit, .. }) => {
                (*progress_limit, *progress_limit, 1.0)
            }
            Some(ProcessingChange::RemoveInvisibleTriangles { progress_limit, .. }) => {
                (*progress_limit, *progress_limit, 1.0)
            }
            Some(ProcessingChange::Extrude { progress_limit, .. }) => {
                (*progress_limit, *progress_limit, 1.0)
            }
            _ => (0, 0, 0.0),
        }
    }

    /// Add map change to the pending queue.
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
        settings: GenerateDensityMeshSettings,
    ) -> Result<(), DensityMapError> {
        self.map.change(col, row, width, height, data)?;
        self.queue.push_back((vec![], settings));
        Ok(())
    }

    /// Process penging change.
    ///
    /// # Returns
    /// Result with process status when ok, otherwise error.
    #[allow(clippy::many_single_char_names)]
    pub fn process(&mut self) -> Result<ProcessStatus, GenerateDensityMeshError> {
        if let Some(current) = self.current.take() {
            match current {
                ProcessingChange::FindingPoints {
                    settings,
                    mut tries,
                    mut remaining,
                    mut points,
                    mut progress_current,
                    progress_limit,
                } => {
                    if !points.is_empty() {
                        remaining = into_iter!(remaining)
                            .filter(|(p1, _, _, lpss)| {
                                points.iter().all(|p2| (*p2 - *p1).sqr_magnitude() > *lpss)
                            })
                            .collect::<Vec<_>>();
                        if remaining.is_empty() {
                            self.current = Some(ProcessingChange::Triangulate {
                                settings,
                                points,
                                progress_limit,
                            });
                            return Ok(ProcessStatus::InProgress);
                        }
                    }
                    if let Some((point, _, _, _)) = remaining
                        .iter()
                        .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                    {
                        points.push(*point);
                        tries = settings.max_iterations;
                    } else if tries > 0 {
                        tries -= 1;
                        self.current = Some(ProcessingChange::FindingPoints {
                            settings,
                            tries,
                            remaining,
                            points,
                            progress_current,
                            progress_limit,
                        });
                        return Ok(ProcessStatus::InProgress);
                    } else {
                        self.current = Some(ProcessingChange::Triangulate {
                            settings,
                            points,
                            progress_limit,
                        });
                        return Ok(ProcessStatus::InProgress);
                    }
                    progress_current = progress_limit - remaining.len();
                    self.current = Some(ProcessingChange::FindingPoints {
                        settings,
                        tries,
                        remaining,
                        points,
                        progress_current,
                        progress_limit,
                    });
                    Ok(ProcessStatus::InProgress)
                }
                ProcessingChange::Triangulate {
                    settings,
                    points,
                    progress_limit,
                } => {
                    let dpoints = points
                        .iter()
                        .map(|v| Point::new(v.x, v.y))
                        .collect::<Vec<_>>();
                    let triangulation = if let Some(triangulation) = Delaunay::new(&dpoints) {
                        triangulation
                    } else {
                        return Err(GenerateDensityMeshError::FailedTriangulation);
                    };
                    let triangles = triangulation
                        .dcel
                        .vertices
                        .chunks(3)
                        .map(|t| Triangle {
                            a: t[0],
                            b: t[1],
                            c: t[2],
                        })
                        .collect::<Vec<_>>();
                    if !settings.keep_invisible_triangles {
                        self.current = Some(ProcessingChange::RemoveInvisibleTriangles {
                            settings,
                            points,
                            triangles,
                            progress_limit,
                        });
                        Ok(ProcessStatus::InProgress)
                    } else if let Some(size) = settings.extrude_size {
                        self.current = Some(ProcessingChange::Extrude {
                            points,
                            triangles,
                            size,
                            progress_limit,
                        });
                        Ok(ProcessStatus::InProgress)
                    } else {
                        self.mesh = Some(DensityMesh { points, triangles });
                        Ok(ProcessStatus::MeshChanged)
                    }
                }
                ProcessingChange::RemoveInvisibleTriangles {
                    settings,
                    points,
                    mut triangles,
                    progress_limit,
                } => {
                    triangles = triangles
                        .into_iter()
                        .filter(|t| {
                            Self::is_triangle_visible(
                                points[t.a],
                                points[t.b],
                                points[t.c],
                                &self.map,
                                &settings,
                            )
                        })
                        .collect::<Vec<_>>();
                    if let Some(size) = settings.extrude_size {
                        self.current = Some(ProcessingChange::Extrude {
                            points,
                            triangles,
                            size,
                            progress_limit,
                        });
                        Ok(ProcessStatus::InProgress)
                    } else {
                        self.mesh = Some(DensityMesh { points, triangles });
                        Ok(ProcessStatus::MeshChanged)
                    }
                }
                ProcessingChange::Extrude {
                    mut points,
                    mut triangles,
                    size,
                    ..
                } => {
                    let (p, t) = Self::extrude(&points, &triangles, size);
                    points.extend(p);
                    triangles.extend(t);
                    self.mesh = Some(DensityMesh { points, triangles });
                    Ok(ProcessStatus::MeshChanged)
                }
            }
        } else if let Some((points, settings)) = self.queue.pop_front() {
            let scale = self.map.scale();
            let remaining = self
                .map
                .value_steepness_iter()
                .filter_map(|(x, y, v, s)| {
                    if v > settings.visibility_threshold && s > settings.steepness_threshold {
                        let x = (x * scale) as Scalar;
                        let y = (y * scale) as Scalar;
                        let lpss = match settings.points_separation {
                            PointsSeparation::Constant(v) => v * v,
                            PointsSeparation::SteepnessMapping(f, t) => {
                                let v = Self::lerp(s, t, f);
                                v * v
                            }
                        };
                        Some((Coord::new(x, y), v, s, lpss))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let progress_limit = remaining.len();
            let tries = settings.max_iterations;
            self.current = Some(ProcessingChange::FindingPoints {
                settings,
                tries,
                remaining,
                points,
                progress_current: 0,
                progress_limit,
            });
            Ok(ProcessStatus::InProgress)
        } else {
            Ok(ProcessStatus::Idle)
        }
    }

    /// Process incoming changes until none is left to do.
    ///
    /// # Returns
    /// Ok or generation error.
    pub fn process_wait(&mut self) -> Result<(), GenerateDensityMeshError> {
        while self.process()? == ProcessStatus::InProgress {}
        Ok(())
    }

    /// Process incoming changes until none is left to do.
    ///
    /// # Arguments
    /// * `timeout` - Duration of time that processing can take.
    ///
    /// # Returns
    /// Process status or generation error.
    pub fn process_wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<ProcessStatus, GenerateDensityMeshError> {
        let timer = Instant::now();
        loop {
            let status = self.process()?;
            if status != ProcessStatus::InProgress || timer.elapsed() > timeout {
                return Ok(status);
            }
        }
    }

    /// Process incoming changes until none is left to do.
    ///
    /// # Arguments
    /// * `f` - Callback triggered on every processing step. Signature: `fn(progress, limit, factor)`.
    ///
    /// # Returns
    /// Ok or generation error.
    pub fn process_wait_tracked<F>(&mut self, mut f: F) -> Result<(), GenerateDensityMeshError>
    where
        F: FnMut(usize, usize, Scalar),
    {
        let (c, l, p) = self.progress();
        f(c, l, p);
        while self.process()? == ProcessStatus::InProgress {
            let (c, l, p) = self.progress();
            f(c, l, p);
        }
        Ok(())
    }

    /// Process incoming changes until none is left to do.
    ///
    /// # Arguments
    /// * `f` - Callback triggered on every processing step. Signature: `fn(progress, limit, factor)`.
    /// * `timeout` - Duration of time that processing can take.
    ///
    /// # Returns
    /// Process status or generation error.
    pub fn process_wait_timeout_tracked<F>(
        &mut self,
        mut f: F,
        timeout: Duration,
    ) -> Result<ProcessStatus, GenerateDensityMeshError>
    where
        F: FnMut(usize, usize, Scalar),
    {
        let timer = Instant::now();
        let (c, l, p) = self.progress();
        f(c, l, p);
        loop {
            let status = self.process()?;
            let (c, l, p) = self.progress();
            f(c, l, p);
            if status != ProcessStatus::InProgress || timer.elapsed() > timeout {
                return Ok(status);
            }
        }
    }

    fn extrude(
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
                    .any(|e2| e1.0 != e2.0 && Self::are_edges_connected(e1.1, e1.2, e2.1, e2.2))
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
                    if Self::is_point_visible((x, y), map, settings) {
                        count += 1;
                    }
                }
            }
        }
        count as Scalar / samples as Scalar > 0.5
    }

    #[inline]
    fn is_point_visible(
        pos: (isize, isize),
        map: &DensityMap,
        settings: &GenerateDensityMeshSettings,
    ) -> bool {
        map.value_at_point(pos) > settings.visibility_threshold
    }

    #[inline]
    fn lerp(value: Scalar, from: Scalar, to: Scalar) -> Scalar {
        from + (to - from) * value.max(0.0).min(1.0)
    }
}
