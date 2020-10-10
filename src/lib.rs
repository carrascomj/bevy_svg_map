use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use roxmltree;
use std::{error::Error, fs};
use svgtypes::{PathParser, PathSegment};

mod style;
pub use style::{StyleStrategy, SvgStyle};

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
pub fn load_svg_map<T: StyleStrategy>(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    svg_map: &str,
    strategy: T,
) {
    // let wall_thickness = 10.0;
    let (x_max, y_max) = tokenize_svg(svg_map)
        .unwrap()
        .iter()
        .flat_map(|n| n.traces.iter())
        .fold((0f64, 0f64), |acc, n| {
            let x_f = match n.x() {
                Some(x) => x.abs().max(acc.0),
                None => acc.0,
            };
            let y_f = match n.y() {
                Some(y) => y.abs().max(acc.1),
                None => acc.1,
            };
            (x_f, y_f)
        });
    let (x_max, y_max) = ((x_max / 2.) as f32, (y_max / 2.) as f32);

    for StyleSegment { style, traces } in tokenize_svg(svg_map).unwrap().iter() {
        let mut origin = Vec3::new(0f32, 0f32, 0f32);
        let color_handle = materials.add(strategy.color_decider(style).into());
        let mut builder = PathBuilder::new();
        for tok in traces.iter() {
            match tok {
                PathSegment::MoveTo { abs: _, x, y } => {
                    let (x, y) = ((*x as f32).abs() - x_max, (*y as f32).abs() - y_max);
                    origin = Vec3::new(x, y, 0f32);
                    builder.move_to(point(x, y));
                }
                PathSegment::LineTo { abs: _, x, y } => {
                    let (x, y) = ((*x as f32).abs() - x_max, (*y as f32).abs() - y_max);
                    origin = Vec3::new(x, y, 0f32);
                    builder.line_to(point(x, y));
                }
                PathSegment::HorizontalLineTo { abs: _, x } => {
                    let x = (*x as f32).abs() - x_max;
                    builder.line_to(point(x, origin.y()));
                    // .with(Collider::Solid);
                    origin = Vec3::new(x, origin.y(), 0f32);
                }
                PathSegment::VerticalLineTo { abs: _, y } => {
                    let y = (*y as f32).abs() - y_max;
                    builder.line_to(point(origin.x(), y));
                    // .with(Collider::Solid);
                    origin = Vec3::new(origin.x(), y, 0f32);
                }
                _ => println!("Found not implemented path"),
            }
        }
        builder.close();
        let path = builder.build();
        strategy.component_decider(
            &style,
            commands.spawn(
                path.stroke(
                    color_handle,
                    &mut meshes,
                    Vec3::new(0.0, 0.0, 0.0),
                    &StrokeOptions::default()
                        .with_line_width(5.0)
                        .with_line_cap(LineCap::Round)
                        .with_line_join(LineJoin::Round),
                ),
            ),
        )
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
