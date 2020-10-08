use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use density_mesh_core::{
    generate_densitymesh_from_points_cloud, generate_densitymesh_from_points_cloud_tracked, Coord,
    DensityMesh, GenerateDensityMeshSettings, Scalar,
};
use density_mesh_image::{
    generate_densitymap_from_image, generate_densitymap_image, GenerateDensityImageSettings,
    ImageDensitySource,
};
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
                    Arg::with_name("is-chunk")
                        .long("is-chunk")
                        .help("Density map is a chunk, part of the bigger density map")
                        .takes_value(false)
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
                .parse::<Scalar>()
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
            let is_chunk = matches.is_present("is-chunk");
            let keep_invisible_triangles = matches.is_present("keep-invisible-triangles");
            let settings = GenerateDensityMeshSettings {
                points_separation,
                visibility_threshold,
                steepness_threshold,
                max_iterations,
                extrude_size,
                is_chunk,
                keep_invisible_triangles,
            };
            if verbose {
                println!("{:#?}", settings);
            }
            let mesh = if verbose {
                generate_densitymesh_from_points_cloud_tracked(
                    vec![],
                    map,
                    settings,
                    |current, limit, percentage| {
                        println!(
                            "Progress: {}% ({} / {})",
                            (percentage * 100.0).max(0.0).min(100.0),
                            current,
                            limit
                        );
                    },
                )
            } else {
                generate_densitymesh_from_points_cloud(vec![], map, settings)
            }
            .expect("Cannot produce density mesh");
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
    fn test_data_image() {
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
    fn test_chunks() {
        let image = DynamicImage::ImageRgba8(
            image::open("../resources/heightmap.png")
                .expect("Cannot open file")
                .to_rgba(),
        );
        let count = 4;
        let width = image.width() / count;
        let height = image.height() / count;
        let images = (0..(count * count))
            .into_iter()
            .map(|i| {
                let col = i % count;
                let row = i / count;
                let x = col * width;
                let y = row * height;
                let mut image = image.crop_imm(x, y, width + 1, height + 1);
                let settings = GenerateDensityImageSettings::default();
                let map = generate_densitymap_from_image(image.clone(), &settings)
                    .expect("Cannot produce density map image");
                let settings = GenerateDensityMeshSettings {
                    points_separation: 16.0,
                    is_chunk: true,
                    keep_invisible_triangles: true,
                    ..Default::default()
                };
                let mesh = generate_densitymesh_from_points_cloud(vec![], map, settings)
                    .expect("Cannot produce density mesh");
                apply_mesh_on_map(&mut image, &mesh);
                (col, row, image)
            })
            .collect::<Vec<_>>();
        let mut image = DynamicImage::new_rgba8(width * count, height * count);
        for (col, row, subimage) in images {
            image
                .copy_from(&subimage, col * width, row * height)
                .expect("Could not copy subimage");
        }
        image
            .save("../resources/heightmap.vis.png")
            .expect("Cannot save output image");
    }
}
