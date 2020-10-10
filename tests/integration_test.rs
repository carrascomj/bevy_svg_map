use bevy_svg_map::{load_svg_map, StyleStrategy, SvgStyle};

use bevy::prelude::*;

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
    fn component_decider(&self, style: &SvgStyle, comp: &mut Commands) {
        comp.with(if style.stroke_opacity().unwrap() == 1.0 {
            Collider::Solid
        } else {
            Collider::Scorable
        });
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

#[test]
fn can_it_be_added() {
    App::build().add_default_plugins().add_plugin(TestPlugin);
}

#[test]
fn custom_style_strategy() {
    App::build()
        .add_default_plugins()
        .add_startup_system(setup_custom.system());
}
