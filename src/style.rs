use bevy::prelude::{Color, Vec3};
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;
use svgtypes::PathSegment;
use svgtypes::{Length, NumberList, Paint};

fn linear_to_nonlinear_srgb(num: f32) -> u8 {
    let res: f32;
    if num <= 0.0 || num == 1. {
        res = num;
    } else if num <= 0.0031308 {
        res = num * 12.92; // linear falloff in dark values
    } else {
        res = (1.055 * num.powf(1.0 / 2.4)) - 0.055 //gamma curve in other area
    }
    (res * 255.0) as u8
}

/// Translater from SVG style (&str slice) to bevy (passing )
/// The string slice is parsed into a HashMap. Lazy accession to its values.
/// Chief struct to implement the user-provided strategy to associate components/materials given
/// the style of the path.
///
/// Except stroke_opacity() and fill_opacity() that return a `Result`, all the types return an
/// `Option<T>`. More properties could be exposed but these are enough for now to build things.
///
/// # Example
///
/// ```
/// use bevy_svg_map::SvgStyle;
/// use bevy::prelude::Color;
///
/// let style = SvgStyle::from("fill:none;stroke:#000000;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1");
/// assert_eq!(
///     style.stroke().unwrap(),
///     Color::BLACK
/// );
/// ```
#[derive(Debug)]
pub struct SvgStyle(HashMap<String, String>);

/// Helper function that transforms from str to svgtypes' Color to bevy's Color
fn to_color(color: &str, opacity: u8) -> Option<Color> {
    println!("{}", opacity);
    if let Ok(Paint::Color(svgtypes::Color { red, green, blue })) = Paint::from_str(color) {
        Some(Color::rgba_u8(red as u8, green as u8, blue as u8, opacity))
    } else {
        None
    }
}

impl SvgStyle {
    pub fn stroke(&self) -> Option<Color> {
        to_color(
            self.panic_access("stroke"),
            match self.stroke_opacity() {
                Ok(c) => linear_to_nonlinear_srgb(c),
                _ => 255,
            },
        )
    }
    pub fn fill(&self) -> Option<Color> {
        to_color(
            self.panic_access("fill"),
            match self.fill_opacity() {
                Ok(c) => linear_to_nonlinear_srgb(c),
                _ => 255,
            },
        )
    }
    pub fn stroke_dashmap(&self) -> Option<NumberList> {
        match self.0.get("stroke-dashmap") {
            Some(c) => Some(c.parse().unwrap()),
            _ => None,
        }
    }
    /// In both opacities, please remember that they return a Result (it may change in the future)
    /// ```
    /// # use bevy_svg_map::SvgStyle;
    /// # use bevy::prelude::Color;
    ///
    /// let style = SvgStyle::from("fill:none;stroke:#000000;stroke-width:1.0px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:0.5");
    /// assert_eq!(
    ///     style.stroke_opacity().unwrap(),
    ///     0.5
    /// );
    /// ```
    pub fn stroke_opacity(&self) -> Result<f32, std::num::ParseFloatError> {
        match self.0.get("stroke-opacity") {
            Some(c) => c.parse(),
            _ => Ok(1f32),
        }
    }
    pub fn fill_opacity(&self) -> Result<f32, std::num::ParseFloatError> {
        match self.0.get("fill-opacity") {
            Some(c) => c.parse(),
            _ => Ok(1f32),
        }
    }
    pub fn stroke_width(&self) -> Option<f32> {
        if let Ok(Length { num, unit: _ }) = Length::from_str(self.panic_access("stroke-width")) {
            Some(num as f32)
        } else {
            None
        }
    }
    fn panic_access(&self, key: &str) -> &str {
        match self.0.get(key) {
            Some(value) => value,
            _ => panic!(
                "Field {} (used to build svg-based components) is missing! Check your SVG file",
                key
            ),
        }
    }
}

impl From<&str> for SvgStyle {
    fn from(style: &str) -> Self {
        SvgStyle(
            style
                .split(';')
                .map(|n| {
                    let a: Vec<&str> = n.split(':').take(2).collect();
                    (a[0].to_string(), a[1].to_string())
                })
                .collect::<HashMap<String, String>>(),
        )
    }
}

impl Default for SvgStyle {
    fn default() -> Self {
        Self::from("fill:none;stroke:#000000;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1")
    }
}

pub trait StyleStrategy {
    fn color_decider(&self, _style: &SvgStyle) -> Color {
        Color::BLACK
    }
    fn component_decider(&self, _style: &SvgStyle, _sprite: &mut bevy::prelude::Commands) {}
}

pub struct SvgWhole;

// TODO implement everything
impl StyleStrategy for SvgWhole {
    fn color_decider(&self, style: &SvgStyle) -> Color {
        match style.stroke() {
            Some(c) => c,
            // add red lines if the Color could not be parsed from the SVG
            _ => Color::RED,
        }
    }
}

pub fn build_svg(
    token: &PathSegment,
    builder: &mut PathBuilder,
    origin: Vec3,
    x_max: f32,
    y_max: f32,
    x_scale: f32,
    y_scale: f32,
) -> Vec3 {
    let transform = |x: &f64, y: &f64| -> (f32, f32) {
        (
            ((*x as f32).abs() - x_max) * x_scale,
            ((*y as f32).abs() - y_max) * y_scale,
        )
    };
    let mut new_origin = Vec3::from(origin);
    match token {
        PathSegment::MoveTo { abs: _, x, y } => {
            let (x, y) = transform(x, y);
            new_origin = Vec3::new(x, y, 0f32);
            builder.move_to(point(x, y));
        }
        PathSegment::LineTo { abs: _, x, y } => {
            let (x, y) = transform(x, y);
            new_origin = Vec3::new(x, y, 0f32);
            builder.line_to(point(x, y));
        }
        PathSegment::HorizontalLineTo { abs: _, x } => {
            let (x, _) = transform(x, &0f64);
            builder.line_to(point(x, origin.y()));
            new_origin = Vec3::new(x, origin.y(), 0f32);
        }
        PathSegment::VerticalLineTo { abs: _, y } => {
            let (_, y) = transform(&0f64, y);
            builder.line_to(point(origin.x(), y));
            new_origin = Vec3::new(origin.x(), y, 0f32);
        }
        PathSegment::Quadratic {
            abs: _,
            x1,
            y1,
            x,
            y,
        } => {
            let (x, y) = transform(x, y);
            let to = point(x, y);
            let control = point((*x1 as f32).abs() - x_max, (*y1 as f32).abs() - y_max);
            builder.quadratic_bezier_to(control, to);
            new_origin = Vec3::new(to.x, to.y, 0f32);
        }
        PathSegment::CurveTo {
            abs: _,
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        } => {
            let (x, y) = transform(x, y);
            let to = point(x, y);
            let (x1, y1) = transform(x1, y1);
            let control1 = point(x1, y1);
            let (x2, y2) = transform(x2, y2);
            let control2 = point(x2, y2);
            builder.cubic_bezier_to(control1, control2, to);
            new_origin = Vec3::new(to.x, to.y, 0f32);
        }
        PathSegment::EllipticalArc {
            abs: _,
            rx,
            ry,
            x_axis_rotation,
            large_arc: _,
            sweep: _,
            x,
            y,
        } => {
            let (x, y) = transform(x, y);
            let center = point(x, y);
            let (rx, ry) = transform(rx, ry);
            builder.arc(
                center,
                rx,
                ry,
                std::f32::consts::PI,
                *x_axis_rotation as f32,
            );
            new_origin = Vec3::new(center.x, center.y, 0f32);
        }
        PathSegment::ClosePath { abs: _ } => builder.close(),
        _ => println!("SVG mapper: Found not implemented path!"),
    }
    new_origin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_parse_default() {
        let style = SvgStyle::default();
        assert_eq!(style.stroke().unwrap(), Color::BLACK);
    }

    #[test]
    fn test_stroke_width() {
        let style = SvgStyle::default();
        assert_eq!(style.stroke_width().unwrap(), 0.264583);
    }
}
