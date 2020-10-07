use density_mesh_core::{DensityMap, DensityMapError, Scalar};
use image::{imageops::FilterType, DynamicImage, GenericImageView, GrayImage};
use serde::{Deserialize, Serialize};

/// Source image preprocessing mode (at the end you get grayscale image representing density map
/// or typically a height map).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageDensitySource {
    /// Luminosity.
    Luma,
    /// Luminosity * Alpha.
    LumaAlpha,
    /// Red channel.
    Red,
    /// Green channel.
    Green,
    /// Blue channel.
    Blue,
    /// Alpha channel.
    Alpha,
}

impl Default for ImageDensitySource {
    fn default() -> Self {
        Self::LumaAlpha
    }
}

/// Settings of density image generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDensityImageSettings {
    /// Image density source.
    #[serde(default)]
    pub density_source: ImageDensitySource,
    /// Scale of the image (image is rescaled to: original size / scale).
    #[serde(default = "GenerateDensityImageSettings::default_scale")]
    pub scale: usize,
}

impl Default for GenerateDensityImageSettings {
    fn default() -> Self {
        Self {
            density_source: ImageDensitySource::default(),
            scale: GenerateDensityImageSettings::default_scale(),
        }
    }
}

impl GenerateDensityImageSettings {
    fn default_scale() -> usize {
        1
    }
}

/// Generate density map image.
///
/// # Arguments
/// * `image` - Input image to process.
/// * `settings` - Image processing settings.
/// * `steepness` - If true produce steepness image, if false produce density image.
///
/// # Returns
/// Processed image or error.
pub fn generate_densitymap_image(
    image: DynamicImage,
    settings: &GenerateDensityImageSettings,
    steepness: bool,
) -> Result<DynamicImage, DensityMapError> {
    let map = generate_densitymap_from_image(image, settings)?;
    let data = if steepness {
        map.steepness()
            .iter()
            .map(|v| (v * 255.0) as u8)
            .collect::<Vec<_>>()
    } else {
        map.values()
            .iter()
            .map(|v| (v * 255.0) as u8)
            .collect::<Vec<_>>()
    };
    Ok(DynamicImage::ImageLuma8(
        GrayImage::from_raw(map.unscaled_width() as _, map.unscaled_height() as _, data).unwrap(),
    ))
}

/// Generate density map from image.
///
/// # Arguments
/// * `image` - Input image to process.
/// * `settings` - Image processing settings.
///
/// # Returns
/// Density map or error.
pub fn generate_densitymap_from_image(
    image: DynamicImage,
    settings: &GenerateDensityImageSettings,
) -> Result<DensityMap, DensityMapError> {
    let scale = settings.scale.max(1);
    let image = if scale > 1 {
        image.resize_exact(
            image.width() / scale as u32,
            image.height() / scale as u32,
            FilterType::Lanczos3,
        )
    } else {
        image
    };
    match settings.density_source {
        ImageDensitySource::Luma => {
            let img = image.to_luma();
            DensityMap::new(img.width() as _, img.height() as _, scale, img.into_raw())
        }
        ImageDensitySource::LumaAlpha => {
            let w = image.width();
            let h = image.height();
            let img = image.to_luma_alpha();
            let data = img
                .into_raw()
                .chunks(2)
                .map(|c| ((c[0] as Scalar / 255.0) * (c[1] as Scalar / 255.0) * 255.0) as u8)
                .collect::<Vec<_>>();
            DensityMap::new(w as _, h as _, scale, data)
        }
        ImageDensitySource::Red => {
            let w = image.width();
            let h = image.height();
            let data = image
                .to_rgba()
                .into_raw()
                .chunks(4)
                .map(|c| c[0])
                .collect::<Vec<_>>();
            DensityMap::new(w as _, h as _, scale, data)
        }
        ImageDensitySource::Green => {
            let w = image.width();
            let h = image.height();
            let data = image
                .to_rgba()
                .into_raw()
                .chunks(4)
                .map(|c| c[1])
                .collect::<Vec<_>>();
            DensityMap::new(w as _, h as _, scale, data)
        }
        ImageDensitySource::Blue => {
            let w = image.width();
            let h = image.height();
            let data = image
                .to_rgba()
                .into_raw()
                .chunks(4)
                .map(|c| c[2])
                .collect::<Vec<_>>();
            DensityMap::new(w as _, h as _, scale, data)
        }
        ImageDensitySource::Alpha => {
            let w = image.width();
            let h = image.height();
            let data = image
                .to_rgba()
                .into_raw()
                .chunks(4)
                .map(|c| c[3])
                .collect::<Vec<_>>();
            DensityMap::new(w as _, h as _, scale, data)
        }
    }
}
