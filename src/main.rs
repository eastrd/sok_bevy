#![allow(unused)]

use bevy::{asset::Asset, prelude::*};
use camera::CameraPlugin;
use debug::DebugPlugin;
use scene::ScenePlugin;

mod camera;
mod data;
mod debug;
mod scene;
mod universe;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
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
        .run();

    // Generate an interconnected universe of stack exchange using Bevy 3D
}
