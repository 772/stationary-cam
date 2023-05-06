# stationary-cam

This safe CLI generates a stationary cam as .svg.

## Build example from source

```
git clone https://github.com/772/stationary-cam
cd stationary-cam/example
cargo r -- example.toml
```

![Result](example/example.png)

## Using [Blender](https://www.blender.org) to convert the svg to 3D

1. File -> Import -> Scalable Vector Graphics (.svg)
1. Select all objects
1. Object Data Properties -> Resolution Preview U -> See console
1. Right click -> Set Origin -> Geometry to Origin
1. Right click -> Convert To -> Mesh
1. Select cam object and select all vertices
1. Press E to move them up a bit
1. Select tooth object and select all vertices
1. ALT-N -> Flip
1. Press E to move them down a bit
1. n times: Select each side or two vertices and merge to last
1. Select all objects
1. Control-M -> z
1. Right click -> Join
1. File -> Export -> e.g. Wavefront (.obj) / .stl or what you wish

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
