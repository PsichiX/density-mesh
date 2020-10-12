pub mod settings;

use crate::settings::{GenerateDensityImageSettings, ImageDensitySource};
use density_mesh_core::{
    map::{DensityMap, DensityMapError},
    Scalar,
};
use image::{imageops::FilterType, DynamicImage, GenericImageView, GrayImage};

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

/// Generate image from density map.
///
/// # Arguments
/// * `map` - Input density map.
/// * `steepness` - If true produce steepness image, if false produce density image.
///
/// # Returns
/// Grayscale image.
pub fn generate_image_from_densitymap(map: &DensityMap, steepness: bool) -> DynamicImage {
    DynamicImage::ImageLuma8(
        GrayImage::from_raw(
            map.unscaled_width() as _,
            map.unscaled_height() as _,
            if steepness {
                map.steepness()
                    .iter()
                    .map(|v| (v * 255.0) as u8)
                    .collect::<Vec<_>>()
            } else {
                map.values()
                    .iter()
                    .map(|v| (v * 255.0) as u8)
                    .collect::<Vec<_>>()
            },
        )
        .unwrap(),
    )
}

pub mod prelude {
    pub use crate::{
        generate_densitymap_from_image, generate_densitymap_image, generate_image_from_densitymap,
        settings::*,
    };
}
