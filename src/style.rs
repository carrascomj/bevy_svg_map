use bevy::prelude::Color;
use lyon::lyon_tessellation::{LineCap, LineJoin};
use std::collections::HashMap;
use std::str::FromStr;
use svgtypes::{Length, NumberList, Paint};

/// Adapted from bevy_render
fn linear_to_nonlinear_srgb(num: f32) -> u8 {
    let res: f32;
    if num <= 0.0 || num >= 0.99 {
        res = num;
    } else if num <= 0.0031308 {
        res = num * 12.92; // linear falloff in dark values
    } else {
        res = (1.055 * num.powf(1.0 / 2.4)) - 0.055 //gamma curve in other area
    }
    (res * 255.0) as u8
}
/// Helper function that transforms from str to svgtypes' Color to bevy's Color
fn to_color(color: &str, opacity: u8) -> Option<Color> {
    if let Ok(Paint::Color(svgtypes::Color { red, green, blue })) = Paint::from_str(color) {
        Some(Color::rgba_u8(red as u8, green as u8, blue as u8, opacity))
    } else {
        None
    }
}

/// Stores the style and the SVG type (later parsed by lyon and svgtypes)
/// It corresponds to a single SpriteComponent
#[derive(Debug)]
pub struct StyleSegment {
    pub style: SvgStyle,
    pub traces: String,
}

impl From<(&str, &str)> for StyleSegment {
    fn from(tup: (&str, &str)) -> Self {
        let style: SvgStyle = SvgStyle::from(tup.0);
        let traces = tup.1.to_string();
        StyleSegment { style, traces }
    }
}

/// Translater from SVG style (&str slice) to bevy
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
    /// The resulting [`svgtypes::NumberList`](https://docs.rs/svgtypes/0.5.0/src/svgtypes/number_list.rs.html)
    /// can be treated as Vec<f64>
    /// See: [<list-of-numbers>](https://www.w3.org/TR/SVG11/types.html#DataTypeList)
    /// ```
    /// # use bevy_svg_map::SvgStyle;
    /// # use bevy::prelude::Color;
    /// use svgtypes::NumberList;
    ///
    /// let style = SvgStyle::from("stroke:#000000;stroke-dasharray:3,1");
    /// assert_eq!(
    ///     style.stroke_dasharray().unwrap()[0],
    ///     3f64
    /// );
    /// ```
    /// It also implements `Iter`
    /// ```
    /// # use bevy_svg_map::SvgStyle;
    /// # use bevy::prelude::Color;
    /// use svgtypes::NumberList;
    ///
    /// let style = SvgStyle::from("stroke:#000000;stroke-dasharray:3,1");
    /// assert_eq!(style.stroke_dasharray().unwrap().iter().sum::<f64>(), 4f64);
    /// ```
    pub fn stroke_dasharray(&self) -> Option<NumberList> {
        match self.0.get("stroke-dasharray") {
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
    /// Parse the string as a Lyon LineCap
    /// See: https://svgwg.org/specs/strokes/#StrokeLinecapProperty
    ///
    /// ```
    /// # use bevy_svg_map::SvgStyle;
    /// # use bevy::prelude::Color;
    /// use lyon::lyon_tessellation::LineCap;
    ///
    /// let style = SvgStyle::from("stroke-linecap:butt;stroke-linejoin:miter");
    /// assert_eq!(
    ///     style.stroke_linecap().unwrap(),
    ///     LineCap::Butt
    /// );
    /// ```
    pub fn stroke_linecap(&self) -> Option<LineCap> {
        match self.0.get("stroke-linecap") {
            Some(c) => match c.as_ref() {
                "butt" => Some(LineCap::Butt),
                "round" => Some(LineCap::Round),
                "square" => Some(LineCap::Square),
                _ => None,
            },
            _ => None,
        }
    }
    /// Parse the string as a Lyon LineJoin
    /// See: https://svgwg.org/specs/strokes/#StrokeLinejoinProperty
    ///
    /// ```
    /// # use bevy_svg_map::SvgStyle;
    /// # use bevy::prelude::Color;
    /// use lyon::lyon_tessellation::LineJoin;
    ///
    /// let style = SvgStyle::from("stroke-linecap:butt;stroke-linejoin:miter");
    /// assert_eq!(
    ///     style.stroke_linejoin().unwrap(),
    ///     LineJoin::Miter
    /// );
    /// ```
    pub fn stroke_linejoin(&self) -> Option<LineJoin> {
        match self.0.get("stroke-linejoin") {
            Some(c) => match c.as_ref() {
                "butt" => Some(LineJoin::Bevel),
                "miter" => Some(LineJoin::Miter),
                "miterclip" => Some(LineJoin::MiterClip),
                "round" => Some(LineJoin::Round),
                _ => None,
            },
            _ => None,
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

/// This trait is implemented by the user as the Strategy to add functionaly to the paths based
/// on its properties (stored in `SvgStyle`).
pub trait StyleStrategy {
    fn color_decider(&self, _style: &SvgStyle) -> Color {
        Color::BLACK
    }
    fn component_decider(&self, _style: &SvgStyle, _sprite: &mut bevy::prelude::Commands) {}
}

/// Used when loading whole SVG files as a single entity.
/// Implements StyleStrategy to literal visual properties.
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
