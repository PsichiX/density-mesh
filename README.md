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

Typical use case would be to use two of them to create mesh from images but in
case you have your own image handler, you can stick to the core module and
produce density maps by yourself.

#### Working with chunks
In the past, there was a way to optimize work with big maps using chunks - these
chunks wasn't giving a reliable topology and had to be removed.

#### Real-time density mesh modifications
Imagine that you have a one big mesh, you want to modify variable size regions
of this mesh and don't want to split it into chunks - for this use case there is
specialized `DensityMeshGenerator` type. Keep in mind that at the moment, even
if you change only really smart part of the map, whole mesh will be rebuilt so
for big maps this might take long which means, for now you shouldn't use this
crate for high performance, HD maps generation.

```rust
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
```

With that we have added two solid rectangles and result looks like:

![image live](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/heightmap.live.png)

#### Optimizations of map region changes
Previous versions had live mesh generator which regenerated only the parts of
the mesh that given region has changed - for now this is not further supported
because of corner case situations where mesh generation crashes internally
making use of this feature unreliable. This feature wil be added later when it
will be redesigned.

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
density-mesh 1.5.0
CLI app for density mesh generator
Patryk 'PsichiX' Budzynski <psichix@gmail.com>

Usage:
  density-mesh <COMMAND>

Commands:
  image  Produce density map image
  mesh   Produce density mesh
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
density-mesh-image
Produce density map image

Usage:
  density-mesh image [OPTIONS] --input <PATH> --output <PATH>

Options:
  -i, --input <PATH>              Input file path
  -o, --output <PATH>             Output file path
      --density-source <CHANNEL>  Use an alternate channel as density source [default: luma-alpha] [possible values: luma, luma-alpha, red, green, blue, alpha]
      --scale <INTEGER>           Image scale [default: 1]
      --verbose                   Display settings used
  -s, --steepness                 Produce steepness image
  -h, --help                      Print help
```

```
density-mesh-mesh
Produce density mesh

Usage:
  density-mesh mesh [OPTIONS] --input <PATH> --output <PATH> <--json|--json-pretty|--yaml|--obj|--png>

Options:
  -i, --input <PATH>
          Input file path
  -o, --output <PATH>
          Output file path
      --density-source <CHANNEL>
          Use an alternate channel as density source [default: luma-alpha] [possible values: luma, luma-alpha, red, green, blue, alpha]
      --scale <INTEGER>
          Image scale [default: 1]
      --verbose
          Display settings used
      --json
          Produce JSON mesh
      --json-pretty
          Produce pretty JSON mesh
      --yaml
          Produce YAML mesh
      --obj
          Produce OBJ mesh
      --png
          Produce PNG mesh visualization
      --points-separation <NUMBER_OR_RANGE>
          Points separation [default: 10]
      --visibility-threshold <NUMBER>
          Visibility threshold [default: 0.01]
      --steepness-threshold <NUMBER>
          Steepness threshold [default: 0.01]
      --max-iterations <INTEGER>
          Maximum number of tries when finding point to place [default: 32]
      --extrude-size <NUMBER>
          Extrude size
      --update-region-margin <NUMBER>
          Margin around update region box; currently unused [default: 0]
      --keep-invisible-triangles
          Keep invisible triangles
  -h, --help
          Print help (see more with '--help')
```
