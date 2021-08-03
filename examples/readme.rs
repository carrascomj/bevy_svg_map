// Example that generates the image in the README

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_svg_map::{load_svg_map, StyleStrategy, SvgStyle};

pub enum Collider {
    Solid,
}

struct CustomStrategy;

impl StyleStrategy for CustomStrategy {
    fn color_decider(&self, style: &SvgStyle) -> Color {
        match style.stroke() {
            Some(c) => c,
            _ => Color::RED,
        }
    }
    fn component_decider(&self, _style: &SvgStyle, comp: &mut EntityCommands) {
        comp.insert(Collider::Solid);
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_svg.system());
    }
}

fn setup_svg(
    commands: Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
) {
    load_svg_map(commands, materials, meshes, "assets/ex.svg", CustomStrategy);
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldPlugin)
        .add_startup_system(setup.system())
        .run();
}

// Add an entity with a camera attached; orange square that could be the player
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("orange_square.png");
    // Player
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
    commands
        .spawn()
        .insert_bundle(camera)
        .insert_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(52., 52.)),
            ..Default::default()
        });
}
