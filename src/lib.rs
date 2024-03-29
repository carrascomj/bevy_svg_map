use bevy::prelude::*;
use euclid::default::Transform2D;
use lyon::svg::path_utils::build_path;
use lyon::tessellation::{FillOptions, StrokeOptions};
use std::{error::Error, fs};
use svgtypes::PathParser;

mod lyon_utils;
mod style;
use style::StyleSegment;
pub use style::{StyleStrategy, SvgStyle};

/// Return a zero-cost read-only view of the svg XML document as a graph
fn take_lines_with_style<'a>(
    doc: &'a roxmltree::Document,
) -> Vec<(&'a str, &'a str, Option<&'a str>, Option<&'a str>)> {
    doc.descendants()
        .filter(|n| matches!(n.attribute("d"), Some(_)))
        .map(|n| {
            (
                n.attribute("style").unwrap(),
                n.attribute("d").unwrap(),
                n.attribute("id"),
                n.attribute("class"),
            )
        })
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
        .flat_map(|n| PathParser::from(n.traces.as_ref()).map(|n| n.unwrap()))
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

/// For each of the paths in a SVG file, apply a StyleStrategy to translate them into entities with
/// functionality added to them, dependent of the SVG properties of the path (stroke, fill...)
pub fn load_svg_map<T: StyleStrategy>(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    svg_map: &str,
    strategy: T,
) {
    let (x_max, y_max) = max_coords(svg_map);
    let (x_max, y_max) = (x_max as f32, y_max as f32);

    for StyleSegment { style, traces } in tokenize_svg(svg_map).unwrap().iter() {
        let color_handle = materials.add(strategy.color_decider(style).into());
        // TODO: this transformation are a joke...
        let builder = lyon::path::Path::builder().with_svg().transformed(
            Transform2D::translation(x_max + x_max / 2f32, y_max / 2f32) // translate to bevy coordinates
                .pre_rotate(euclid::Angle::radians(std::f32::consts::PI / 2.)) // rotate 180º for some reason
                .then(&Transform2D::new(0f32, 1f32, 1f32, 0f32, 0f32, 0f32)) // mirror for some reason
                .then_translate(euclid::Vector2D::new(0., -y_max)), // translate again to bevy coordinates
        );
        let path = build_path(builder, traces).unwrap();
        if matches!(style.stroke(), Some(_)) {
            strategy.component_decider(
                &style,
                commands.spawn().insert_bundle(lyon_utils::stroke(
                    path.clone(),
                    color_handle.clone(),
                    &mut meshes,
                    Vec3::new(-x_max, -y_max, 0.0),
                    &StrokeOptions::default()
                        .with_line_width(strategy.width_decider(style))
                        .with_line_cap(strategy.linecap_decider(style))
                        .with_line_join(strategy.linejoin_decider(style)),
                )),
            )
        }
        if matches!(style.fill(), Some(_)) {
            strategy.component_decider(
                &style,
                commands.spawn().insert_bundle(lyon_utils::fill(
                    path,
                    color_handle,
                    &mut meshes,
                    Vec3::new(-x_max, -y_max, 0.0),
                    &FillOptions::default(),
                )),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svgtypes::PathSegment;
    #[test]
    fn tokenize_properly() {
        let (_, _) = tokenize_svg("assets/ex.svg")
            .unwrap()
            .iter()
            .flat_map(|n| PathParser::from(n.traces.as_ref()).map(|n| n.unwrap()))
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
    #[test]
    fn tokenize_id_attribute() {
        assert!(tokenize_svg("assets/ex.svg")
            .unwrap()
            .iter()
            .any(|st| st.style.id().is_some()));
    }
}
