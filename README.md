# Image density/height map to mesh generator

![crates-io version](https://raster.shields.io/crates/v/density-mesh-core.png)

## About
Crates used to generate 2D mesh from images representing density/height map.

**Algorithm gets source image:**

![image source](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/logo.png)

**Converts it into density/height values (here from alpha channell):**

![image values](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/logo.data.png)

**Next makes steepness values:**

![image steepness](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/logo.steepness.png)

**And builds mesh based on the highest steepness points:**

![image mesh](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/logo.vis.png)

## Rust API
#### Important modules
- https://crates.io/crates/density-mesh-core - Module used to generate mesh from density map.
- https://crates.io/crates/density-mesh-image - Module used to generate density map from image.

Typical use case would be to use two of them to create mesh from images but in case you have your own image handler, you can stick to the core module and produce density maps by yourself.

#### Working with chunks
Chunks are used for example for real-time terrain generation when you have a destructible heightmap and just want to update modified chunks at a time, instead of whole terrain.

```rust
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
```

## CLI
#### Install / Update
```bash
cargo install density-mesh-cli --force
```

#### Example
```bash
density-mesh mesh -i image.png -o mesh.obj --obj
```

#### Options
```
density-mesh-cli 1.0.0
Patryk 'PsichiX' Budzynski <psichix@gmail.com>
CLI app for density mesh generator

USAGE:
    density-mesh [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    image    Produce density map image
    mesh     Produce density mesh
```

```
density-mesh-image
Produce density map image

USAGE:
    density-mesh image [FLAGS] [OPTIONS] --input <PATH> --output <PATH>

FLAGS:
    -h, --help         Prints help information
    -s, --steepness    Produce steepness image
    -V, --version      Prints version information
        --verbose      Display settings used

OPTIONS:
        --density-source <NAME>    Density source: luma, luma-alpha, red, green, blue, alpha [default: luma-alpha]
    -i, --input <PATH>             Input image file
    -o, --output <PATH>            Output image file
        --scale <INTEGER>          Image scale [default: 1]
```

```
density-mesh-mesh
Produce density mesh

USAGE:
    density-mesh mesh [FLAGS] [OPTIONS] --input <PATH> --output <PATH> <--json|--json-pretty|--yaml|--obj|--png>

FLAGS:
    -h, --help           Prints help information
        --json           Produce JSON mesh
        --json-pretty    Produce pretty JSON mesh
        --obj            Produce OBJ mesh
        --png            Produce PNG mesh visualization
    -V, --version        Prints version information
        --verbose        Display settings used
        --yaml           Produce YAML mesh

OPTIONS:
        --density-source <NAME>            Density source: luma, luma-alpha, red, green, blue, alpha [default: luma-
                                           alpha]
        --extrude-size <NUMBER>            Extrude size
    -i, --input <PATH>                     Input image file
        --max-iterations <INTEGER>         Maximum tries number when finding point to place [default: 32]
    -o, --output <PATH>                    Output mesh file
        --points-separation <NUMBER>       Points separation [default: 10]
        --scale <INTEGER>                  Image scale [default: 1]
        --steepness-threshold <NUMBER>     Steepness threshold [default: 0.01]
        --visibility-threshold <NUMBER>    VIsibility threshold [default: 0.01]
```
