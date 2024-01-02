// mod game;
// mod setup;

mod generation;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use generation::GenerationPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin, GenerationPlugin))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup)
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
