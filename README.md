# Image density/height map to mesh generator

![crates-io version](https://raster.shields.io/crates/v/density-mesh-core.png)

## About
Crates used to generate 2D mesh from images representing density/height map.

Algorithm gets source image:
![image source](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/logo.png)

Converts it into density/height values:
![image values](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/logo.data.png)

Next makes steepness values:
![image steepness](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/logo.steepness.png)

And builds mesh based on the highest steepness points:
![image mesh](https://raw.githubusercontent.com/PsichiX/density-mesh/master/resources/logo.vis.png)

## CLI
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
