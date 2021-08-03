<p align="left">
<img height=180 src="./assets/bevy_svg_map_logo.svg" alt="Project logo">
</p>

![build](https://github.com/carrascomj/bevy_svg_map/workflows/build/badge.svg?branch=master)
[![Build Status](https://img.shields.io/crates/v/bevy_svg_map.svg)](https://crates.io/crates/bevy_svg_map/)
[![](https://docs.rs/bevy_svg_map/badge.svg)](https://docs.rs/bevy_svg_map)

# Bevy SVG map


Crate for loading SVG files directly into [bevy](https://github.com/bevyengine/bevy/).
The properties of the lines (color, opacity, fill…) can be used to programmatically
add functionality, setting the foundation for a _very_ weird workflow: Vector Graphics as editor to bevy entities!

> If you are looking for a plugin that loads SVGs, you may want to take a look at [bevy_svg](https://github.com/Weasy666/bevy_svg).

## Getting started
Add the library to your project's `Cargo.toml` (check last published version,
it corresponds to the stable version on the branch [`crates.io`](https://github.com/carrascomj/bevy_svg_map/tree/crates-io)):
```toml
[dependencies]
bevy_svg_map = "0.2"
```
If you want the bleeding edge (corresponds to [`master`](https://github.com/carrascomj/bevy_svg_map/tree/master/)):
```toml
[dependencies]
bevy_svg_map = {git="https://github.com/carrascomj/bevy_svg_map.git"}
```
> :warning: The master branch points to each master branch of **bevy**. Use with caution!

The library provides a function to be used inside a bevy's startup_system.
Here, we are loading the file `ex.svg` under the `assets/` directory.

```rust
use bevy_svg_map::load_svg_map;
use bevy::prelude::*;

// We need to provide a struct implementing the StyleStrategy
// leave it as default, we'll come back to this later
struct MyStrategy;

impl StyleStrategy for MyStrategy {}

fn main() {
    App::build()
          .add_default_plugins()
          .add_startup_system(setup.system())
          .run();
}

fn setup(mut com: Commands, mat: ResMut<Assets<ColorMaterial>>, mesh: ResMut<Assets<Mesh>>) {
    load_svg_map(com, mat, mesh, "assets/ex.svg", MyStrategy);
}
```
That should display some lines as in the image on the top. However, they are plain
black (default for `StyleStrategy`). What about using the colors from the SVG
path strokes?
```rust
// we're now also using SvgStyle
use bevy_svg_map::{load_svg_map, StyleStrategy, SvgStyle};
use bevy::prelude::*;

struct MyStrategy;

impl StyleStrategy for MyStrategy {
  // implement this trait method
  fn color_decider(&self, style: &SvgStyle) -> Color {
        match style.stroke() {
            Some(c) => c,
            // add red lines if the Color could not be parsed from the SVG
            _ => Color::RED,
        }
    }
}
```
OK, that's a bit more interesting. Notice how `SvgStyle` exposes properties of
the style of the SVG path. For each of these paths, the `color_decider`
function is applied to… well… decide its color.

Finally, to provide some actually interesting functionality, we can apply
arbitrary functions to each component created from the path.

```rust
// ... same as before
use bevy::ecs::system::EntityCommands;

// This Component will be added to each SpriteComponents created from a path
enum Collider {
    Scorable,
    Solid,
}

impl StyleStrategy for MyStrategy {
  // implement this trait method
  fn color_decider(&self, style: &SvgStyle) -> Color {
        // same as before
  }
  fn component_decider(&self, style: &SvgStyle, mut comp: EntityCommands) {
    // use the stroke opacity to decide the kind of Collider
      comp.insert(
          if style.stroke_opacity().unwrap() == 1.0 {
              Collider::Solid
          } else{
              Collider::Scorable
          }
      );
  }
}
```

Check out more properties to extract from `SvgStyle` in the documentation!

## Troubleshooting
* Set up your Document Properties (in Inkscape _Ctrl+Shift+D_) to pixels so that you get the right world units.
* See [this comment](https://github.com/carrascomj/bevy_svg_map/issues/1#issuecomment-706611397) about setting SVG output.

## Features
* [x] Load Horizontal and Vertical lines.
* [x] Load other types of [svgtypes](https://github.com/RazrFalcon/svgtypes) [`PathSegment`s]().
* [x] Provide a [strategy](https://en.wikipedia.org/wiki/Strategy_pattern) trait
to use the style to add Components and materials.
* [ ] Basic shapes.
* [ ] Handling of units.
