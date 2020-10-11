use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::{error::Error, fs};
use svgtypes::{PathParser, PathSegment};

mod style;
use style::{build_svg, SvgWhole};
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
        .filter(|n| matches!(n.attribute("d"), Some(_)))
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

fn max_coords(svg_map: &str) -> (f64, f64) {
    tokenize_svg(svg_map)
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
        })
}

fn max_coords_doc(svg_map: &str) -> (f64, f64) {
    let xmlfile = fs::read_to_string(svg_map).unwrap();
    let doc = roxmltree::Document::parse(&xmlfile).unwrap();
    (
        doc.descendants()
            .filter(|n| n.tag_name().name() == "svg")
            .map(|n| n.attribute("width").unwrap().parse().unwrap())
            .last()
            .unwrap(),
        doc.descendants()
            .filter(|n| n.tag_name().name() == "svg")
            .map(|n| n.attribute("height").unwrap().parse().unwrap())
            .last()
            .unwrap(),
    )
}

/// For each of the paths in a SVG file, apply a StyleStrategy to translate them into entities with
/// functionality added to them, dependent of the SVG properties of the path (stroke, fill...)
pub fn load_svg_map<T: StyleStrategy>(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    svg_map: &str,
    strategy: T,
) {
    // let wall_thickness = 10.0;
    let (x_max, y_max) = max_coords(svg_map);
    let (x_max, y_max) = (x_max / 2., y_max / 2.);

    for StyleSegment { style, traces } in tokenize_svg(svg_map).unwrap().iter() {
        let mut origin = Vec3::new(0f32, 0f32, 0f32);
        let color_handle = materials.add(strategy.color_decider(style).into());
        let mut builder = PathBuilder::new();
        for tok in traces.iter() {
            origin = build_svg(
                tok,
                &mut builder,
                origin,
                x_max as f32,
                y_max as f32,
                1.,
                1.,
            );
        }
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

/// Load a SVG file as an Entity, return the Commands to allot the user to further modify it
pub fn load_svg(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    svg_map: &str,
    width: f32,
    height: f32,
) -> Commands {
    let (x_in, y_in) = max_coords_doc(svg_map);
    let (x_max, y_max) = max_coords(svg_map);
    let (x_max, y_max) = (x_max / 2., y_max / 2.);
    let (scale_x, scale_y) = ((width / x_in as f32), (height / y_in as f32));

    let mut origin = Vec3::new(0f32, 0f32, 0f32);
    // let mut sprites = Vec::new();
    commands.spawn((Transform::default(), GlobalTransform::default()));
    let parent = commands.current_entity().unwrap();
    for StyleSegment { style, traces } in tokenize_svg(svg_map).unwrap().iter() {
        let mut builder = PathBuilder::new();
        let color_handle = materials.add(SvgWhole.color_decider(style).into());
        for tok in traces.iter() {
            origin = build_svg(
                tok,
                &mut builder,
                origin,
                x_max as f32,
                y_max as f32,
                scale_x,
                // given the default Transform and the modifications in build_svg, this is the
                // way of getting it right
                -scale_y,
            );
        }
        let path = builder.build();
        let sprite = path.stroke(
            color_handle,
            &mut meshes,
            Vec3::new(0.0, 0.0, 0.0),
            &StrokeOptions::default()
                .with_line_width(5.0)
                .with_line_cap(LineCap::Round)
                .with_line_join(LineJoin::Round),
        );
        commands.spawn(sprite);
        let sprite1 = commands.current_entity().unwrap();
        commands.push_children(parent, &[sprite1]);
    }
    commands
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
