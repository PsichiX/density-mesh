use density_mesh_core::prelude::*;
use image::*;
use minifb::*;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, time::Duration};

const WIDTH: usize = 256;
const HEIGHT: usize = 256;
const BRUSH_SIZE: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    current: DensityMeshGenerator,
    prev: Option<DensityMeshGenerator>,
    next: Option<DensityMeshGenerator>,
}

fn main() {
    let options = WindowOptions {
        scale: Scale::X4,
        ..Default::default()
    };
    let mut window = Window::new("Density Mesh Playground", WIDTH, HEIGHT, options)
        .expect("Could not create window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut last_pos = None;
    let map = DensityMap::new(WIDTH, HEIGHT, 1, vec![127; WIDTH * HEIGHT]).unwrap();
    let settings = GenerateDensityMeshSettings {
        points_separation: (5.0, 10.0).into(),
        keep_invisible_triangles: true,
        ..Default::default()
    };
    let mut generator = DensityMeshGenerator::new(vec![], map.clone(), settings.clone());
    generator.process_wait().expect("Processing failed");
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
    let mut history = VecDeque::<DensityMeshGenerator>::new();
    history.push_back(generator.clone());
    let mut restore = VecDeque::<DensityMeshGenerator>::new();
    let mut time_min_max = None;
    let mut dirty = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            let state = State {
                current: generator.clone(),
                prev: history.back().cloned(),
                next: restore.back().cloned(),
            };
            match serde_json::to_string(&state) {
                Ok(content) => match std::fs::write("./resources/save.json", content) {
                    Ok(_) => println!("* State saved!"),
                    Err(error) => println!("* Cannot save state: {:?}", error),
                },
                Err(error) => println!("* Cannot serialize state: {:?}", error),
            }
        }
        if window.is_key_pressed(Key::L, KeyRepeat::No) {
            match std::fs::read_to_string("./resources/save.json") {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(state) => {
                        let State {
                            current,
                            prev,
                            next,
                        } = state;
                        history.clear();
                        restore.clear();
                        generator = current;
                        if let Some(generator) = prev {
                            history.push_back(generator);
                        }
                        if let Some(generator) = next {
                            restore.push_back(generator);
                        }
                        println!("* State loaded!");
                    }
                    Err(error) => println!("* Cannot deserialize state: {:?}", error),
                },
                Err(error) => println!("* Cannot serialize state: {:?}", error),
            }
        }
        if window.is_key_pressed(Key::C, KeyRepeat::No) {
            generator = DensityMeshGenerator::new(vec![], map.clone(), settings.clone());
            history.clear();
            history.push_back(generator.clone());
            restore.clear();
        }
        if window.is_key_pressed(Key::Z, KeyRepeat::Yes) {
            if let Some(h) = history.pop_back() {
                restore.push_back(std::mem::replace(&mut generator, h));
            }
        }
        if window.is_key_pressed(Key::X, KeyRepeat::Yes) {
            if let Some(r) = restore.pop_back() {
                history.push_back(std::mem::replace(&mut generator, r));
            }
        }
        let mouse_left = window.get_mouse_down(MouseButton::Left);
        let mouse_right = window.get_mouse_down(MouseButton::Right);
        let pause = mouse_left || mouse_right;
        if pause {
            if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
                let x = x as usize;
                let y = y as usize;
                let allow = if let Some((lx, ly)) = last_pos {
                    lx != x || ly != y
                } else {
                    true
                };
                if allow {
                    while history.len() >= 100 {
                        history.pop_front();
                    }
                    history.push_back(generator.clone());
                    restore.clear();
                    paint(&mut generator, x, y, &brush, mouse_left, &settings);
                    dirty = true;
                }
                last_pos = Some((x, y));
            }
        } else {
            last_pos = None;
        }
        if dirty {
            let timer = std::time::Instant::now();
            dirty = generator
                .process_wait_timeout(Duration::from_millis(16))
                .expect("Processing failed")
                == ProcessStatus::InProgress;
            let elapsed = timer.elapsed();
            time_min_max = match time_min_max {
                Some((min, max)) => Some((elapsed.min(min), elapsed.max(max))),
                None => Some((elapsed, elapsed)),
            };
            if dirty {
                window.update();
            } else {
                let data = generator
                    .map()
                    .values()
                    .iter()
                    .map(|v| (v * 255.0) as u8)
                    .collect::<Vec<_>>();
                let image = DynamicImage::ImageLuma8(
                    GrayImage::from_raw(WIDTH as u32, HEIGHT as u32, data).unwrap(),
                );
                let mut image = DynamicImage::ImageRgba8(image.into_rgba());
                apply_generator_on_map(&mut image, &generator, [0, 255, 0, 255]);
                let buffer = image
                    .pixels()
                    .map(|(_, _, pixel)| {
                        let [r, g, b, _] = pixel.0;
                        (b as u32) | ((g as u32) << 8) | ((r as u32) << 16)
                    })
                    .collect::<Vec<_>>();
                window
                    .update_with_buffer(&buffer, WIDTH, HEIGHT)
                    .expect("Could not update window buffer");
            }
        } else {
            window.update();
        }
    }
    if let Some((min, max)) = time_min_max {
        println!("GENERATION TIME: {:?} -> {:?}", min, max);
    }
}

#[allow(clippy::many_single_char_names)]
fn paint(
    generator: &mut DensityMeshGenerator,
    x: usize,
    y: usize,
    brush: &[u8],
    additive: bool,
    settings: &GenerateDensityMeshSettings,
) {
    let half_size = BRUSH_SIZE / 2;
    let x = x.saturating_sub(half_size).min(WIDTH - BRUSH_SIZE - 1);
    let y = y.saturating_sub(half_size).min(HEIGHT - BRUSH_SIZE - 1);
    let data = (0..(BRUSH_SIZE * BRUSH_SIZE))
        .map(|i| {
            let b = brush[i];
            let dc = i % BRUSH_SIZE;
            let dr = i / BRUSH_SIZE;
            let sc = x + dc;
            let sr = y + dr;
            let i = sr * WIDTH + sc;
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

fn apply_generator_on_map(
    image: &mut DynamicImage,
    generator: &DensityMeshGenerator,
    color: [u8; 4],
) {
    if let Some(mesh) = generator.mesh() {
        apply_mesh_on_map(image, mesh, color);
    }
}

fn apply_mesh_on_map(image: &mut DynamicImage, mesh: &DensityMesh, color: [u8; 4]) {
    for triangle in &mesh.triangles {
        let a = mesh.points[triangle.a];
        let b = mesh.points[triangle.b];
        let c = mesh.points[triangle.c];
        apply_line_on_map(image, a, b, color);
        apply_line_on_map(image, b, c, color);
        apply_line_on_map(image, c, a, color);
        let p = (a + b + c) / 3.0;
        apply_point_on_map(image, p, color);
    }
}

fn apply_point_on_map(image: &mut DynamicImage, point: Coord, color: [u8; 4]) {
    let x = point.x as isize;
    let y = point.y as isize;
    if x >= 0 && x < image.width() as _ && y >= 0 && y < image.height() as _ {
        image.put_pixel(x as _, y as _, color.into());
    }
}

fn apply_line_on_map(image: &mut DynamicImage, from: Coord, to: Coord, color: [u8; 4]) {
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
                image.put_pixel(x as _, y as _, color.into());
            }
        }
    } else {
        let (fy, ty, fx, _) = paired_min_max(fy, ty, fx, tx);
        for y in fy..ty {
            let f = (y - fy) as Scalar / dy;
            let x = fx + (dx * f) as isize;
            if x >= 0 && x < image.width() as _ && y >= 0 && y < image.height() as _ {
                image.put_pixel(x as _, y as _, color.into());
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
