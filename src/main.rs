#![allow(unused)]

use bevy::{asset::Asset, prelude::*};
use camera::CameraPlugin;
use scene::ScenePlugin;

mod camera;
mod scene;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.7, 0.7, 1.)))
        .insert_resource(WindowDescriptor {
            title: "SO Bevy".to_string(),
            width: 3440.0 / 2.,
            height: 1440.0 / 2.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ScenePlugin)
        .add_plugin(CameraPlugin)
        .run();
}
