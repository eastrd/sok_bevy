#![allow(unused)]

use bevy::prelude::*;
use camera::CameraPlugin;
use debug::DebugPlugin;
use scene::ScenePlugin;

mod camera;
mod data;
mod debug;
mod scene;
mod universe;

struct WinSize {
    pub w: f32,
    pub h: f32,
}

// Generate an interconnected universe of stack exchange using Bevy 3D
fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .insert_resource(WindowDescriptor {
            title: "Stack Exchange Cartography".to_string(),
            width: 3440.0 / 2.,
            height: 1440.0 / 2.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ScenePlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(DebugPlugin)
        .add_startup_system(setup_window_size)
        .run();
}

fn setup_window_size(mut commands: Commands, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();

    commands.insert_resource(WinSize {
        w: window.width(),
        h: window.height(),
    });
}
