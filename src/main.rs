mod fruit;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite_render::{Wireframe2dConfig, Wireframe2dPlugin};

use crate::fruit::FruitsPlugin;

fn toggle_wireframe(
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        Wireframe2dPlugin::default(),
        PhysicsPlugins::default(),
    ));
    app.add_systems(Update, toggle_wireframe);
    app.add_plugins(FruitsPlugin);
    app.run();
}
