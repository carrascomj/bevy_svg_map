use bevy_svg_map::{load_svg, load_svg_map, StyleStrategy, SvgStyle};

use bevy::{ecs::system::EntityCommands, prelude::*};

struct MyStrategy;

impl StyleStrategy for MyStrategy {}

struct CustomStrategy;

enum Collider {
    Scorable,
    Solid,
}

impl StyleStrategy for CustomStrategy {
    fn color_decider(&self, style: &SvgStyle) -> Color {
        match style.stroke() {
            Some(c) => c,
            _ => Color::RED,
        }
    }
    fn component_decider(&self, style: &SvgStyle, comp: &mut EntityCommands) {
        comp.insert(if style.stroke_opacity().unwrap() == 1.0 {
            Collider::Solid
        } else {
            Collider::Scorable
        });
    }

    fn color_fill_decider(&self, _style: &SvgStyle) -> Color {
        Color::BLACK
    }

    fn width_decider(&self, style: &SvgStyle) -> f32 {
        match style.stroke_width() {
            Some(c) => c,
            _ => 0.264583,
        }
    }

    fn linecap_decider(&self, style: &SvgStyle) -> lyon::lyon_tessellation::LineCap {
        match style.stroke_linecap() {
            Some(c) => c,
            _ => lyon::lyon_tessellation::LineCap::Butt,
        }
    }

    fn linejoin_decider(&self, style: &SvgStyle) -> lyon::lyon_tessellation::LineJoin {
        match style.stroke_linejoin() {
            Some(c) => c,
            _ => lyon::lyon_tessellation::LineJoin::Miter,
        }
    }
}

struct TestPlugin;
impl Plugin for TestPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system());
    }
}

fn setup(
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    load_svg_map(commands, materials, meshes, "assets/ex.svg", MyStrategy);
}

fn setup_custom(
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    load_svg_map(commands, materials, meshes, "assets/ex.svg", CustomStrategy);
}

fn setup_whole_svg(
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    load_svg(
        commands,
        materials,
        meshes,
        "assets/ex.svg",
        1.,
        2.,
        Vec2::new(0., 0.),
    );
}

fn setup_with_shapes(
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    load_svg(
        commands,
        materials,
        meshes,
        "assets/with_shapes.svg",
        64.,
        64.,
        Vec2::new(0., 0.),
    );
}

#[test]
fn can_it_be_added() {
    App::build().add_plugin(TestPlugin);
}

#[test]
fn custom_style_strategy() {
    App::build().add_startup_system(setup_custom.system());
}

#[test]
fn whole_svg() {
    App::build().add_startup_system(setup_whole_svg.system());
}

#[test]
fn whole_svg_with_shapes() {
    App::build().add_startup_system(setup_with_shapes.system());
}
