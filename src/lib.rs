use bevy::prelude::*;
use roxmltree;
use std::{error::Error, fs};
use svgtypes::{PathParser, PathSegment};

mod style;
pub use style::SvgStyle;

/// Struct that parses the svg paths with their style
#[derive(Debug)]
struct StyleSegment {
    style: SvgStyle,
    traces: Vec<PathSegment>,
}

impl From<(&str, &str)> for StyleSegment {
    fn from(tup: (&str, &str)) -> Self {
        let style: SvgStyle = SvgStyle::from(tup.0);
        let traces = PathParser::from(tup.1).map(|n| n.unwrap()).collect();
        StyleSegment { style, traces }
    }
}

/// Return a zero-cost read-only view of the svg XML document as a graph
fn take_lines_with_style<'a>(doc: &'a roxmltree::Document) -> Vec<(&'a str, &'a str)> {
    doc.descendants()
        .filter(|n| match n.attribute("d") {
            Some(_) => true,
            _ => false,
        })
        .map(|n| (n.attribute("style").unwrap(), n.attribute("d").unwrap()))
        .collect()
}

/// Parse each "d" node's attribute into a StyleSegment
fn tokenize_svg(path: &str) -> Result<Vec<StyleSegment>, Box<dyn Error>> {
    let xmlfile = fs::read_to_string(path)?;
    let doc = roxmltree::Document::parse(&xmlfile)?;
    Ok(take_lines_with_style(&doc)
        .iter()
        .map(|p| StyleSegment::from(*p))
        .collect())
}

/// Take the Commands and add Components given the paths in a SVG file
/// TODO: strategy design: expose a trait with a method that returns materials given style,
/// and a method that adds Components given the style.
pub fn load_svg_map(
    mut commands: Commands,
    svg_map: &str,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let wall_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    let wall_thickness = 10.0;
    let (x_max, y_max) = tokenize_svg(svg_map)
        .unwrap()
        .iter()
        .flat_map(|n| n.traces.iter())
        .fold((0f64, 0f64), |acc, n| match n {
            PathSegment::MoveTo { abs: _, x, y } => (x.abs().max(acc.0), y.abs().max(acc.1)),
            PathSegment::HorizontalLineTo { abs: _, x } => (x.abs().max(acc.0), acc.1),
            PathSegment::VerticalLineTo { abs: _, y } => (acc.0, y.abs().max(acc.1)),
            _ => {
                println!("Found a not yet handled PathSegment");
                acc
            }
        });
    let (x_max, y_max) = ((x_max / 2.) as f32, (y_max / 2.) as f32);

    for path in tokenize_svg(svg_map).unwrap().iter() {
        let mut origin = Vec3::new(0f32, 0f32, 0f32);
        println!("{:?}", path.style);
        for tok in path.traces.iter() {
            match tok {
                PathSegment::MoveTo { abs: _, x, y } => {
                    origin = Vec3::new((*x as f32).abs(), (*y as f32).abs(), 0f32);
                    println!("{:?}", origin);
                    continue;
                }
                PathSegment::HorizontalLineTo { abs: _, x } => {
                    let x = (*x as f32).abs();
                    commands.spawn(SpriteComponents {
                        material: wall_material,
                        transform: Transform::from_translation(Vec3::new(
                            (origin.x() + x) / 2.0 - x_max,
                            origin.y() - y_max,
                            0.0,
                        )),
                        sprite: Sprite::new(Vec2::new((origin.x() - x).abs(), wall_thickness)),
                        ..Default::default()
                    });
                    // .with(Collider::Solid);
                    origin = Vec3::new(x, origin.y(), 0f32);
                }
                PathSegment::VerticalLineTo { abs: _, y } => {
                    let y = (*y as f32).abs();
                    commands.spawn(SpriteComponents {
                        material: wall_material,
                        transform: Transform::from_translation(Vec3::new(
                            origin.x() - x_max,
                            (origin.y() + y) / 2.0 - y_max,
                            0.0,
                        )),
                        sprite: Sprite::new(Vec2::new(wall_thickness, (origin.y() - y).abs())),
                        ..Default::default()
                    });
                    // .with(Collider::Solid);
                    origin = Vec3::new(origin.x(), y, 0f32);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokenize_properly() {
        let (_, _) = tokenize_svg("assets/ex.svg")
            .unwrap()
            .iter()
            .flat_map(|n| n.traces.iter())
            .fold((0f64, 0f64), |acc, n| match n {
                PathSegment::MoveTo { abs: _, x, y } => (x.abs().max(acc.0), y.abs().max(acc.1)),
                PathSegment::HorizontalLineTo { abs: _, x } => (x.abs().max(acc.0), acc.1),
                PathSegment::VerticalLineTo { abs: _, y } => (acc.0, y.abs().max(acc.1)),
                _ => {
                    println!("Found a not yet handled PathSegment");
                    acc
                }
            });
    }
}
