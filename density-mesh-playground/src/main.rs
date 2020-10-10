use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1024;
const HEIGHT: usize = 1024;

fn main() {
    let buffer = vec![0u32; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Density Mesh Playground",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Could not create window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut dirty = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if dirty {
            dirty = false;
            window
                .update_with_buffer(&buffer, WIDTH, HEIGHT)
                .expect("Could not update window buffer");
        } else {
            window.update();
        }
    }
}
