use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use density_mesh_core::prelude::*;
use density_mesh_image::prelude::*;
use image::{DynamicImage, GenericImage, GenericImageView};
use obj_exporter::{Geometry, ObjSet, Object, Primitive, Shape, TVertex, Vertex};
use std::fs::write;

fn main() {
    run_app(make_app().get_matches());
}

fn make_app<'a, 'b>() -> App<'a, 'b> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("image")
                .about("Produce density map image")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .value_name("PATH")
                        .help("Input image file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("PATH")
                        .help("Output image file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .help("Display settings used")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("steepness")
                        .short("s")
                        .long("steepness")
                        .help("Produce steepness image")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("density-source")
                        .long("density-source")
                        .value_name("NAME")
                        .help("Density source: luma, luma-alpha, red, green, blue, alpha")
                        .default_value("luma-alpha")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("scale")
                        .long("scale")
                        .value_name("INTEGER")
                        .help("Image scale")
                        .default_value("1")
                        .takes_value(true)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("mesh")
                .about("Produce density mesh")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .value_name("PATH")
                        .help("Input image file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("PATH")
                        .help("Output mesh file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .help("Display settings used")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("json")
                        .long("json")
                        .help("Produce JSON mesh")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("json-pretty")
                        .long("json-pretty")
                        .help("Produce pretty JSON mesh")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("yaml")
                        .long("yaml")
                        .help("Produce YAML mesh")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("obj")
                        .long("obj")
                        .help("Produce OBJ mesh")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("png")
                        .long("png")
                        .help("Produce PNG mesh visualization")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("density-source")
                        .long("density-source")
                        .value_name("NAME")
                        .help("Density source: luma, luma-alpha, red, green, blue, alpha")
                        .default_value("luma-alpha")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("scale")
                        .long("scale")
                        .value_name("INTEGER")
                        .help("Image scale")
                        .default_value("1")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("points-separation")
                        .long("points-separation")
                        .value_name("NUMBER")
                        .help("Points separation")
                        .default_value("10")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("visibility-threshold")
                        .long("visibility-threshold")
                        .value_name("NUMBER")
                        .help("VIsibility threshold")
                        .default_value("0.01")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("steepness-threshold")
                        .long("steepness-threshold")
                        .value_name("NUMBER")
                        .help("Steepness threshold")
                        .default_value("0.01")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("max-iterations")
                        .long("max-iterations")
                        .value_name("INTEGER")
                        .help("Maximum tries number when finding point to place")
                        .default_value("32")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("extrude-size")
                        .long("extrude-size")
                        .value_name("NUMBER")
                        .help("Extrude size")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("update-region-margin")
                        .long("update-region-margin")
                        .value_name("NUMBER")
                        .help("Margin around update region box")
                        .default_value("0")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("keep-invisible-triangles")
                        .long("keep-invisible-triangles")
                        .help("Keep invisible triangles")
                        .takes_value(false)
                        .required(false),
                )
                .group(
                    ArgGroup::with_name("formats")
                        .arg("json")
                        .arg("json-pretty")
                        .arg("yaml")
                        .arg("obj")
                        .arg("png")
                        .required(true),
                ),
        )
}

fn run_app(matches: ArgMatches) {
    match matches.subcommand() {
        ("image", Some(matches)) => {
            let input = matches.value_of("input").unwrap();
            let output = matches.value_of("output").unwrap();
            let verbose = matches.is_present("verbose");
            let steepness = matches.is_present("steepness");
            let density_source = match matches.value_of("density-source").unwrap() {
                "luma" => ImageDensitySource::Luma,
                "luma-alpha" => ImageDensitySource::LumaAlpha,
                "red" => ImageDensitySource::Red,
                "green" => ImageDensitySource::Green,
                "blue" => ImageDensitySource::Blue,
                "alpha" => ImageDensitySource::Alpha,
                id => panic!("Unsupported value: {}", id),
            };
            let scale = matches
                .value_of("scale")
                .unwrap()
                .parse::<usize>()
                .expect("Could not parse integer");
            let settings = GenerateDensityImageSettings {
                density_source,
                scale,
            };
            if verbose {
                println!("{:#?}", settings);
            }
            let image = image::open(input).expect("Cannot open input image");
            let image = generate_densitymap_image(image, &settings, steepness)
                .expect("Cannot produce density map image");
            image.save(output).expect("Cannot save output image");
        }
        ("mesh", Some(matches)) => {
            let input = matches.value_of("input").unwrap();
            let output = matches.value_of("output").unwrap();
            let verbose = matches.is_present("verbose");
            let json = matches.is_present("json");
            let json_pretty = matches.is_present("json-pretty");
            let yaml = matches.is_present("yaml");
            let obj = matches.is_present("obj");
            let png = matches.is_present("png");
            let density_source = match matches.value_of("density-source").unwrap() {
                "luma" => ImageDensitySource::Luma,
                "luma-alpha" => ImageDensitySource::LumaAlpha,
                "red" => ImageDensitySource::Red,
                "green" => ImageDensitySource::Green,
                "blue" => ImageDensitySource::Blue,
                "alpha" => ImageDensitySource::Alpha,
                id => panic!("Unsupported value: {}", id),
            };
            let scale = matches
                .value_of("scale")
                .unwrap()
                .parse::<usize>()
                .expect("Could not parse integer");
            let settings = GenerateDensityImageSettings {
                density_source,
                scale,
            };
            if verbose {
                println!("{:#?}", settings);
            }
            let image = image::open(input).expect("Cannot open input image");
            let width = image.width();
            let height = image.height();
            let map = generate_densitymap_from_image(image.clone(), &settings)
                .expect("Cannot produce density map image");
            let points_separation = matches
                .value_of("points-separation")
                .unwrap()
                .parse::<PointsSeparation>()
                .expect("Could not parse number");
            let visibility_threshold = matches
                .value_of("visibility-threshold")
                .unwrap()
                .parse::<Scalar>()
                .expect("Could not parse number");
            let steepness_threshold = matches
                .value_of("steepness-threshold")
                .unwrap()
                .parse::<Scalar>()
                .expect("Could not parse number");
            let max_iterations = matches
                .value_of("max-iterations")
                .unwrap()
                .parse::<usize>()
                .expect("Could not parse integer");
            let extrude_size = matches
                .value_of("extrude-size")
                .map(|v| v.parse::<Scalar>().expect("Could not parse number"));
            let keep_invisible_triangles = matches.is_present("keep-invisible-triangles");
            let settings = GenerateDensityMeshSettings {
                points_separation,
                visibility_threshold,
                steepness_threshold,
                max_iterations,
                extrude_size,
                keep_invisible_triangles,
            };
            if verbose {
                println!("{:#?}", settings);
            }
            let mut generator = DensityMeshGenerator::new(vec![], map, settings);
            if verbose {
                generator
                    .process_wait_tracked(|current, limit, percentage| {
                        println!(
                            "Progress: {}% ({} / {})",
                            (percentage * 100.0).max(0.0).min(100.0),
                            current,
                            limit
                        );
                    })
                    .expect("Cannot produce density mesh");
            } else {
                generator
                    .process_wait()
                    .expect("Cannot produce density mesh");
            }
            let mesh = generator.into_mesh().expect("Cannot produce density mesh");
            if json {
                let contents = serde_json::to_string(&mesh).expect("Could not serialize JSON mesh");
                write(output, contents).expect("Could not save mesh file");
            } else if json_pretty {
                let contents = serde_json::to_string_pretty(&mesh)
                    .expect("Could not serialize pretty JSON mesh");
                write(output, contents).expect("Could not save mesh file");
            } else if yaml {
                let contents = serde_yaml::to_string(&mesh).expect("Could not serialize YAML mesh");
                write(output, contents).expect("Could not save mesh file");
            } else if obj {
                let object = Object {
                    name: "mesh".to_owned(),
                    vertices: mesh
                        .points
                        .iter()
                        .map(|p| Vertex {
                            x: p.x as _,
                            y: p.y as _,
                            z: 0.0,
                        })
                        .collect::<Vec<_>>(),
                    tex_vertices: mesh
                        .points
                        .iter()
                        .map(|p| TVertex {
                            u: p.x as f64 / width as f64,
                            v: p.y as f64 / height as f64,
                            w: 0.0,
                        })
                        .collect::<Vec<_>>(),
                    normals: vec![Vertex {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    }],
                    geometry: vec![Geometry {
                        material_name: None,
                        shapes: mesh
                            .triangles
                            .iter()
                            .map(|t| Shape {
                                primitive: Primitive::Triangle(
                                    (t.a, Some(t.a), Some(0)),
                                    (t.b, Some(t.b), Some(0)),
                                    (t.c, Some(t.c), Some(0)),
                                ),
                                groups: vec![],
                                smoothing_groups: vec![],
                            })
                            .collect::<Vec<_>>(),
                    }],
                };
                let objects = ObjSet {
                    material_library: None,
                    objects: vec![object],
                };
                obj_exporter::export_to_file(&objects, output).expect("Cannot save mesh file");
            } else if png {
                let mut image = DynamicImage::ImageRgba8(image.to_rgba());
                apply_mesh_on_map(&mut image, &mesh);
                image.save(output).expect("Cannot save output image");
            }
        }
        _ => unreachable!(),
    }
}

fn apply_mesh_on_map(image: &mut DynamicImage, mesh: &DensityMesh) {
    for triangle in &mesh.triangles {
        let a = mesh.points[triangle.a];
        let b = mesh.points[triangle.b];
        let c = mesh.points[triangle.c];
        apply_line_on_map(image, a, b);
        apply_line_on_map(image, b, c);
        apply_line_on_map(image, c, a);
        let p = (a + b + c) / 3.0;
        image.put_pixel(p.x as _, p.y as _, [0, 0, 255, 255].into());
    }
    for p in &mesh.points {
        let x = p.x as isize;
        let y = p.y as isize;
        if x >= 0 && x < image.width() as _ && y >= 0 && y < image.height() as _ {
            image.put_pixel(x as _, y as _, [255, 0, 0, 255].into());
        }
    }
}

fn apply_line_on_map(image: &mut DynamicImage, from: Coord, to: Coord) {
    let fx = from.x as isize;
    let fy = from.y as isize;
    let tx = to.x as isize;
    let ty = to.y as isize;
    let dx = tx - fx;
    let dy = ty - fy;
    if dx == 0 && dy == 0 {
        return;
    }
    let w = dx.abs();
    let h = dy.abs();
    let dx = dx as Scalar;
    let dy = dy as Scalar;
    if w > h {
        let (fx, tx, fy, _) = paired_min_max(fx, tx, fy, ty);
        for x in fx..tx {
            let f = (x - fx) as Scalar / dx;
            let y = fy + (dy * f) as isize;
            if x >= 0 && x < image.width() as _ && y >= 0 && y < image.height() as _ {
                image.put_pixel(x as _, y as _, [0, 255, 0, 255].into());
            }
        }
    } else {
        let (fy, ty, fx, _) = paired_min_max(fy, ty, fx, tx);
        for y in fy..ty {
            let f = (y - fy) as Scalar / dy;
            let x = fx + (dx * f) as isize;
            if x >= 0 && x < image.width() as _ && y >= 0 && y < image.height() as _ {
                image.put_pixel(x as _, y as _, [0, 255, 0, 255].into());
            }
        }
    }
}

fn paired_min_max(a1: isize, b1: isize, a2: isize, b2: isize) -> (isize, isize, isize, isize) {
    if a1 < b1 {
        (a1, b1, a2, b2)
    } else {
        (b1, a1, b2, a2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        run_app(make_app().get_matches_from(vec![
            "density-mesh",
            "image",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.data.png",
            "--density-source",
            "alpha",
        ]));
        run_app(make_app().get_matches_from(vec![
            "density-mesh",
            "image",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.steepness.png",
            "-s",
            "--density-source",
            "alpha",
        ]));
        run_app(make_app().get_matches_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.json",
            "--json",
            "--density-source",
            "alpha",
        ]));
        run_app(make_app().get_matches_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.pretty.json",
            "--json-pretty",
            "--density-source",
            "alpha",
        ]));
        run_app(make_app().get_matches_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.yaml",
            "--yaml",
            "--density-source",
            "alpha",
        ]));
        run_app(make_app().get_matches_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.obj",
            "--obj",
            "--density-source",
            "alpha",
        ]));
        run_app(make_app().get_matches_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.vis.png",
            "--png",
            "--density-source",
            "alpha",
        ]));
    }

    #[test]
    fn test_live() {
        const BRUSH_SIZE: usize = 64;

        fn paint(
            generator: &mut DensityMeshGenerator,
            x: usize,
            y: usize,
            brush: &[u8],
            additive: bool,
            settings: &GenerateDensityMeshSettings,
        ) {
            let half_size = BRUSH_SIZE / 2;
            let x = x
                .checked_sub(half_size)
                .unwrap_or(0)
                .min(generator.map().unscaled_width() - BRUSH_SIZE - 1);
            let y = y
                .checked_sub(half_size)
                .unwrap_or(0)
                .min(generator.map().unscaled_height() - BRUSH_SIZE - 1);
            let data = (0..(BRUSH_SIZE * BRUSH_SIZE))
                .map(|i| {
                    let b = brush[i];
                    let dc = i % BRUSH_SIZE;
                    let dr = i / BRUSH_SIZE;
                    let sc = x + dc;
                    let sr = y + dr;
                    let i = sr * generator.map().unscaled_width() + sc;
                    let v = (generator.map().values()[i] * 255.0) as u8;
                    if additive {
                        v.checked_add(b).unwrap_or(255)
                    } else {
                        v.checked_sub(b).unwrap_or(0)
                    }
                })
                .collect::<Vec<_>>();
            generator
                .change_map(x, y, BRUSH_SIZE, BRUSH_SIZE, data, settings.clone())
                .expect("Cannot change density map");
        }

        let brush = {
            let half_size = BRUSH_SIZE / 2;
            (0..(BRUSH_SIZE * BRUSH_SIZE))
                .map(|i| {
                    let x = (i % BRUSH_SIZE) as Scalar / half_size as Scalar - 1.0;
                    let y = (i / BRUSH_SIZE) as Scalar / half_size as Scalar - 1.0;
                    let o = 1.0 - Coord::new(x, y).magnitude().min(1.0);
                    (o * 255.0) as u8
                })
                .collect::<Vec<_>>()
        };
        let image = DynamicImage::ImageRgba8(
            image::open("../resources/heightmap.png")
                .expect("Cannot open file")
                .to_rgba(),
        );
        let settings = GenerateDensityImageSettings::default();
        let map = generate_densitymap_from_image(image.clone(), &settings)
            .expect("Cannot produce density map image");
        let settings = GenerateDensityMeshSettings {
            points_separation: 16.0.into(),
            keep_invisible_triangles: true,
            ..Default::default()
        };
        let mut generator = DensityMeshGenerator::new(vec![], map, settings.clone());
        generator
            .process_wait()
            .expect("Cannot process generator changes");
        paint(&mut generator, 100, 100, &brush, true, &settings);
        generator
            .process_wait()
            .expect("Cannot process generator changes");
        for i in (0)..(5) {
            let i = 64 + i * 8;
            paint(&mut generator, i, i, &brush, true, &settings);
            generator
                .process_wait()
                .expect("Cannot process generator changes");
        }
        let mut image = image_from_map(generator.map());
        apply_mesh_on_map(&mut image, generator.mesh().unwrap());
        image
            .save("../resources/heightmap.vis.png")
            .expect("Cannot save output image");
        let image = generate_image_from_densitymap(generator.map(), false);
        image
            .save("../resources/heightmap.data.png")
            .expect("Cannot save output image");
        let image = generate_image_from_densitymap(generator.map(), true);
        image
            .save("../resources/heightmap.steepness.png")
            .expect("Cannot save output image");
    }

    #[test]
    fn test_readme() {
        let image = DynamicImage::ImageRgba8(
            image::open("../resources/heightmap.png")
                .expect("Cannot open file")
                .to_rgba(),
        );
        let settings = GenerateDensityImageSettings::default();
        let map = generate_densitymap_from_image(image.clone(), &settings)
            .expect("Cannot produce density map image");
        let settings = GenerateDensityMeshSettings {
            points_separation: 16.0.into(),
            keep_invisible_triangles: true,
            ..Default::default()
        };
        let mut generator = DensityMeshGenerator::new(vec![], map, settings.clone());
        generator.process_wait().expect("Cannot process changes");
        generator
            .change_map(64, 64, 128, 128, vec![255; 128 * 128], settings.clone())
            .expect("Cannot change live mesh map region");
        generator
            .process_wait()
            .expect("Cannot process live changes");
        generator
            .change_map(384, 384, 64, 64, vec![0; 64 * 64], settings)
            .expect("Cannot change live mesh map region");
        generator
            .process_wait()
            .expect("Cannot process live changes");
        let mut image = DynamicImage::ImageRgba8(
            generate_image_from_densitymap(generator.map(), false).to_rgba(),
        );
        apply_mesh_on_map(&mut image, generator.mesh().unwrap());
        image
            .save("../resources/heightmap.live.png")
            .expect("Cannot save output image");
    }

    fn image_from_map(map: &DensityMap) -> DynamicImage {
        DynamicImage::ImageRgba8(generate_image_from_densitymap(map, false).to_rgba())
    }
}
