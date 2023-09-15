mod cli;

use clap::Parser;
use density_mesh_core::prelude::*;
use density_mesh_image::prelude::*;
use image::{DynamicImage, GenericImage};
use obj_exporter::{Geometry, ObjSet, Object, Primitive, Shape, TVertex, Vertex};
use std::fs::write;

use crate::cli::{Action, CliArgs, CommonArgs};

fn main() {
    let args = CliArgs::parse();

    match args.action {
        Action::Image {
            common:
                CommonArgs {
                    input,
                    output,
                    density_source,
                    scale,
                    verbose,
                },
            steepness,
        } => {
            let settings = GenerateDensityImageSettings {
                density_source: density_source.into(),
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
        Action::Mesh {
            common:
                CommonArgs {
                    input,
                    output,
                    density_source,
                    scale,
                    verbose,
                },
            format,
            points_separation,
            visibility_threshold,
            steepness_threshold,
            max_iterations,
            extrude_size,
            update_region_margin: _,
            keep_invisible_triangles,
        } => {
            let settings = GenerateDensityImageSettings {
                density_source: density_source.into(),
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

            if format.json {
                let contents = serde_json::to_string(&mesh).expect("Could not serialize JSON mesh");
                write(output, contents).expect("Could not save mesh file");
            } else if format.json_pretty {
                let contents = serde_json::to_string_pretty(&mesh)
                    .expect("Could not serialize pretty JSON mesh");
                write(output, contents).expect("Could not save mesh file");
            } else if format.yaml {
                let contents = serde_yaml::to_string(&mesh).expect("Could not serialize YAML mesh");
                write(output, contents).expect("Could not save mesh file");
            } else if format.obj {
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
            } else if format.png {
                let mut image = DynamicImage::ImageRgba8(image.to_rgba8());
                apply_mesh_on_map(&mut image, &mesh);
                image.save(output).expect("Cannot save output image");
            }
        }
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
        CliArgs::parse_from(vec![
            "density-mesh",
            "image",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.data.png",
            "--density-source",
            "alpha",
        ]);
        CliArgs::parse_from(vec![
            "density-mesh",
            "image",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.steepness.png",
            "-s",
            "--density-source",
            "alpha",
        ]);
        CliArgs::parse_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.json",
            "--json",
            "--density-source",
            "alpha",
        ]);
        CliArgs::parse_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.pretty.json",
            "--json-pretty",
            "--density-source",
            "alpha",
        ]);
        CliArgs::parse_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.yaml",
            "--yaml",
            "--density-source",
            "alpha",
        ]);
        CliArgs::parse_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.obj",
            "--obj",
            "--density-source",
            "alpha",
        ]);
        CliArgs::parse_from(vec![
            "density-mesh",
            "mesh",
            "-i",
            "../resources/logo.png",
            "-o",
            "../resources/logo.vis.png",
            "--png",
            "--density-source",
            "alpha",
        ]);
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
                .saturating_sub(half_size)
                .min(generator.map().unscaled_width() - BRUSH_SIZE - 1);
            let y = y
                .saturating_sub(half_size)
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
                        v.saturating_add(b)
                    } else {
                        v.saturating_sub(b)
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
                .to_rgba8(),
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
        for i in 0..5 {
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
                .to_rgba8(),
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
            generate_image_from_densitymap(generator.map(), false).to_rgba8(),
        );
        apply_mesh_on_map(&mut image, generator.mesh().unwrap());
        image
            .save("../resources/heightmap.live.png")
            .expect("Cannot save output image");
    }

    fn image_from_map(map: &DensityMap) -> DynamicImage {
        DynamicImage::ImageRgba8(generate_image_from_densitymap(map, false).to_rgba8())
    }
}
