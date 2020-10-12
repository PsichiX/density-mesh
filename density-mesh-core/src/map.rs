use crate::Scalar;
use serde::{Deserialize, Serialize};

/// Error thrown during density map generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DensityMapError {
    /// Wrong data length.
    /// (provided, expected)
    WrongDataLength(usize, usize),
}

/// Density map that contains density data and steepness per pixel.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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
    /// use density_mesh_core::prelude::*;
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
    /// use density_mesh_core::prelude::*;
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

    pub fn crop(&self, col: usize, row: usize, width: usize, height: usize) -> Self {
        let fx = col.min(self.width);
        let fy = row.min(self.height);
        let tx = (col + width).min(self.width);
        let ty = (row + height).min(self.height);
        let w = tx - fx;
        let h = ty - fy;
        let data = (0..(w * h))
            .map(|i| {
                let x = fx + i % w;
                let y = fy + i / w;
                self.data[y * self.width + x]
            })
            .collect::<Vec<_>>();
        let steepness = (0..(w * h))
            .map(|i| {
                let x = fx + i % w;
                let y = fy + i / w;
                self.steepness[y * self.width + x]
            })
            .collect::<Vec<_>>();
        Self {
            width: w,
            height: h,
            scale: self.scale,
            data,
            steepness,
        }
    }

    pub fn change(
        &mut self,
        col: usize,
        row: usize,
        width: usize,
        height: usize,
        data: Vec<u8>,
    ) -> Result<(), DensityMapError> {
        if col == 0 && row == 0 && width == self.width && height == self.height {
            *self = Self::new(width, height, self.scale, data)?;
            Ok(())
        } else if data.len() == width * height {
            for (i, v) in data.into_iter().enumerate() {
                let x = col + i % width;
                let y = row + i / width;
                self.data[y * self.width + x] = v as Scalar / 255.0;
            }
            let fx = col.checked_sub(1).unwrap_or(col);
            let fy = row.checked_sub(1).unwrap_or(row);
            let tx = (col + width + 1).min(self.width);
            let ty = (row + height + 1).min(self.height);
            for row in fy..ty {
                for col in fx..tx {
                    let mut result = 0.0;
                    {
                        let col = col as isize;
                        let row = row as isize;
                        for x in (col - 1)..(col + 1) {
                            for y in (row - 1)..(row + 1) {
                                let a = Self::raw_value(x, y, self.width, self.height, &self.data);
                                let b =
                                    Self::raw_value(x + 1, y, self.width, self.height, &self.data);
                                let c = Self::raw_value(
                                    x + 1,
                                    y + 1,
                                    self.width,
                                    self.height,
                                    &self.data,
                                );
                                let d =
                                    Self::raw_value(x, y + 1, self.width, self.height, &self.data);
                                let ab = (a - b).abs();
                                let cd = (c - d).abs();
                                let ac = (a - c).abs();
                                let bd = (b - d).abs();
                                let ad = (a - d).abs();
                                let bc = (b - c).abs();
                                result += (ab + cd + ac + bd + ad + bc) / 12.0;
                            }
                        }
                    }
                    self.steepness[row * self.width + col] = result;
                }
            }
            Ok(())
        } else {
            Err(DensityMapError::WrongDataLength(data.len(), width * height))
        }
    }

    fn raw_value(x: isize, y: isize, w: usize, h: usize, data: &[Scalar]) -> Scalar {
        if x >= 0 && x < w as _ && y >= 0 && y < h as _ {
            data[y as usize * w + x as usize]
        } else {
            0.0
        }
    }
}
