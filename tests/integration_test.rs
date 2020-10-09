use bevy_svg_map::load_svg_map;

use bevy::app::App;
use bevy::prelude::*;
use bevy::AddDefaultPlugins;

struct TestPlugin;
impl Plugin for TestPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system());
    }
}

fn setup(commands: Commands, materials: ResMut<Assets<ColorMaterial>>) {
    load_svg_map(commands, "assets/ex.svg", materials);
}

#[test]
fn can_it_be_added() {
    App::build().add_default_plugins().add_plugin(TestPlugin);
}
