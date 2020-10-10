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
