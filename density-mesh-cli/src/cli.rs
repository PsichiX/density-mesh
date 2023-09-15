use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use density_mesh_core::{prelude::PointsSeparation, Scalar};
use density_mesh_image::settings::ImageDensitySource;

#[derive(Clone, Debug, Parser)]
#[command(name = "density-mesh", version, author, about)]
#[command(help_template = "\
{name} {version}
{about}
{author}

{usage-heading}
{tab}{usage}

{all-args}
")]
pub struct CliArgs {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Clone, Debug, Args)]
pub struct CommonArgs {
    /// Input file path
    #[arg(short, long, value_name = "PATH", value_hint(ValueHint::FilePath))]
    pub input: PathBuf,

    /// Output file path
    #[arg(short, long, value_name = "PATH", value_hint(ValueHint::FilePath))]
    pub output: PathBuf,

    /// Use an alternate channel as density source
    #[arg(long, value_name = "CHANNEL", default_value_t)]
    pub density_source: DensitySourceSelection,

    /// Image scale
    #[arg(long, value_name = "INTEGER", default_value_t = 1)]
    pub scale: usize,

    /// Display settings used
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Clone, Copy, Debug, Default, strum::Display, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub enum DensitySourceSelection {
    Luma,
    #[default]
    LumaAlpha,
    Red,
    Green,
    Blue,
    Alpha,
}
impl From<DensitySourceSelection> for ImageDensitySource {
    fn from(value: DensitySourceSelection) -> Self {
        use DensitySourceSelection as S; // source
        use ImageDensitySource as T; // target
        match value {
            S::Luma => T::Luma,
            S::LumaAlpha => T::LumaAlpha,
            S::Red => T::Red,
            S::Green => T::Green,
            S::Blue => T::Blue,
            S::Alpha => T::Alpha,
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Action {
    /// Produce density map image
    #[command(help_template = "\
{name}
{about}

{usage-heading}
{tab}{usage}

{all-args}
")]
    Image {
        #[command(flatten)]
        common: CommonArgs,

        /// Produce steepness image
        #[arg(short, long)]
        steepness: bool,
    },

    /// Produce density mesh
    #[command(help_template = "\
{name}
{about}

{usage-heading}
{tab}{usage}

{all-args}
")]
    Mesh {
        #[command(flatten)]
        common: CommonArgs,

        #[command(flatten)]
        format: Format,

        /// Points separation
        ///
        /// Accepts either a single number, or a range in the format of `MIN..MAX`
        ///
        /// - Single number: constant separation
        /// - Range: separation varies depending on steepness
        #[arg(long, value_name = "NUMBER_OR_RANGE", default_value_t = PointsSeparation::Constant(10.0))]
        points_separation: PointsSeparation,

        /// Visibility threshold
        #[arg(long, value_name = "NUMBER", default_value_t = 0.01)]
        visibility_threshold: Scalar,

        /// Steepness threshold
        #[arg(long, value_name = "NUMBER", default_value_t = 0.01)]
        steepness_threshold: Scalar,

        /// Maximum number of tries when finding point to place
        #[arg(long, value_name = "INTEGER", default_value_t = 32)]
        max_iterations: usize,

        /// Extrude size
        #[arg(long, value_name = "NUMBER")]
        extrude_size: Option<Scalar>,

        /// Margin around update region box; currently unused
        #[arg(long, value_name = "NUMBER", default_value_t = 0.0)]
        update_region_margin: Scalar,

        /// Keep invisible triangles
        #[arg(long)]
        keep_invisible_triangles: bool,
    },
}

#[derive(Clone, Debug, Args)]
#[group(required = true)]
pub struct Format {
    /// Produce JSON mesh
    #[arg(long)]
    pub json: bool,

    /// Produce pretty JSON mesh
    #[arg(long)]
    pub json_pretty: bool,

    /// Produce YAML mesh
    #[arg(long)]
    pub yaml: bool,

    /// Produce OBJ mesh
    #[arg(long)]
    pub obj: bool,

    /// Produce PNG mesh visualization
    #[arg(long)]
    pub png: bool,
}
