use bevy::prelude::Color;
use std::collections::HashMap;
use std::str::FromStr;
use svgtypes::{Length, NumberList, Paint};

#[derive(Debug)]
pub struct SvgStyle(HashMap<String, String>);

/// Helper function that transforms from str to svgtypes' Color to bevy's Color
fn to_color(color: &str, opacity: f32) -> Option<Color> {
    if let Ok(Paint::Color(svgtypes::Color { red, green, blue })) = Paint::from_str(color) {
        Some(Color {
            r: red as f32,
            g: green as f32,
            b: blue as f32,
            a: opacity,
        })
    } else {
        None
    }
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
///     Color {
///         r: 0f32,
///         g: 0f32,
///         b: 0f32,
///         a: 1f32
///     }
/// );
/// ```
impl SvgStyle {
    pub fn stroke(&self) -> Option<Color> {
        to_color(
            self.panic_access("stroke"),
            match self.stroke_opacity() {
                Ok(c) => c,
                _ => 1f32,
            },
        )
    }
    pub fn fill(&self) -> Option<Color> {
        to_color(
            self.panic_access("fill"),
            match self.fill_opacity() {
                Ok(c) => c,
                _ => 1f32,
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
                .split(";")
                .map(|n| {
                    let a: Vec<&str> = n.split(":").take(2).collect();
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
        Color {
            r: 0f32,
            g: 0f32,
            b: 0f32,
            a: 1f32,
        }
    }
    fn component_decider(&self, _style: &SvgStyle, _sprite: &mut bevy::prelude::Commands) {
        ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_parse_default() {
        let style = SvgStyle::default();
        assert_eq!(
            style.stroke().unwrap(),
            Color {
                r: 0f32,
                g: 0f32,
                b: 0f32,
                a: 1f32
            }
        );
    }

    #[test]
    fn test_stroke_width() {
        let style = SvgStyle::default();
        assert_eq!(
            style.stroke().unwrap(),
            Color {
                r: 0f32,
                g: 0f32,
                b: 0f32,
                a: 1f32
            }
        );
    }
}
