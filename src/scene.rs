use std::{collections::HashMap, fmt::Debug};

use bevy::prelude::*;
use rand::{thread_rng, Rng};

const FONT_SIZE_DEFAULT: f32 = 10.;
const RANDOM_SPACE_LIMIT: f32 = 10000.;
const PLANET_RADIUS: f32 = 50.;
const PLANET_SUBDIVISIONS: usize = 1;
const FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";
const FONT_COLOR: Color = Color::GOLD;
const LABEL_FADE_DISTANCE: f32 = 4000.;

use crate::{
    camera::SceneCam,
    universe::{generate_universe_cartography, Galaxy, Planet},
};

use bevy_render::camera::Camera;

#[derive(Component)]
struct PlanetComp;

#[derive(Component)]
struct PlanetLabel;

#[derive(Component)]
struct ConnectionComp;

struct CartographyRes {
    planets: HashMap<String, Planet>,
    galaxies: HashMap<String, Galaxy>,
}

#[derive(Debug)]
struct Index {
    label_to_planet: HashMap<Entity, Entity>,
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        let (galaxies, planets) = generate_universe_cartography();
        app.insert_resource(CartographyRes { planets, galaxies })
            .insert_resource(Index {
                label_to_planet: HashMap::new(),
            })
            .add_startup_system(setup_universe)
            .add_system(update_text_position)
            .add_system(update_text_visibility);
        // .add_system(update_text_scale); // <- Too laggy, need to optimize performance first
    }
}

fn update_text_position(
    windows: Res<Windows>,
    mut text_q: Query<(Entity, &mut Style), With<PlanetLabel>>,
    planet_q: Query<&GlobalTransform, With<PlanetComp>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<SceneCam>>,
    index: Res<Index>,
) {
    for (camera, cam_transform) in camera_q.iter() {
        for (text_entity, mut style) in text_q.iter_mut() {
            // Use index to query planet by its text label for performance saving
            let planet_entity = index.label_to_planet.get(&text_entity).unwrap();
            let planet_transform = planet_q.get(*planet_entity).unwrap();

            if let Some(coords) =
                camera.world_to_screen(&windows, cam_transform, planet_transform.translation)
            {
                style.position.left = Val::Px(coords.x);
                style.position.bottom = Val::Px(coords.y);
            }
        }
    }
}

fn update_text_visibility(
    mut text_q: Query<(Entity, &mut Text, &mut Visibility), With<PlanetLabel>>,
    planet_q: Query<&GlobalTransform, With<PlanetComp>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<SceneCam>>,
    index: Res<Index>,
) {
    // Update text label scale to reflect distance
    for (camera, cam_transform) in camera_q.iter() {
        for (text_entity, mut text, mut text_visibility) in text_q.iter_mut() {
            let planet_entity = index.label_to_planet.get(&text_entity).unwrap();
            let planet_transform = planet_q.get(*planet_entity).unwrap();

            // As text is on the UI,
            // need the corresponding planetary data for distance calculation
            let dist = planet_transform
                .translation
                .distance(cam_transform.translation);

            if dist >= LABEL_FADE_DISTANCE {
                text_visibility.is_visible = false;
            } else {
                text_visibility.is_visible = true;
            }
            // let new_font_size = RANDOM_SPACE_LIMIT / dist * FONT_SIZE_DEFAULT;
            // if new_font_size > 1. {
            //     text.sections[0].style.font_size = new_font_size;
            // }
        }
    }
}

fn update_text_scale(
    mut text_q: Query<(Entity, &mut Text), With<PlanetLabel>>,
    planet_q: Query<&GlobalTransform, With<PlanetComp>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<SceneCam>>,
    index: Res<Index>,
) {
    // Update text label scale to reflect distance
    for (camera, cam_transform) in camera_q.iter() {
        for (text_entity, mut text) in text_q.iter_mut() {
            let planet_entity = index.label_to_planet.get(&text_entity).unwrap();
            let planet_transform = planet_q.get(*planet_entity).unwrap();

            // As text is on the UI,
            // need the corresponding planetary data for distance calculation
            let dist = planet_transform
                .translation
                .distance(cam_transform.translation);
            let new_font_size = RANDOM_SPACE_LIMIT / dist * FONT_SIZE_DEFAULT;
            if new_font_size > 1. {
                text.sections[0].style.font_size = new_font_size;
            }
        }
    }
}

fn setup_universe(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cartography: Res<CartographyRes>,
    mut mapping: ResMut<Index>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = thread_rng();
    for (planet_name, planet) in &cartography.planets {
        // choose a random position between 0 and RANDOM_SPACE_LIMIT
        let x: f32 = rng.gen_range(0.0..RANDOM_SPACE_LIMIT);
        let y: f32 = rng.gen_range(0.0..RANDOM_SPACE_LIMIT);
        let z: f32 = rng.gen_range(0.0..RANDOM_SPACE_LIMIT);

        let planet_id = commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: PLANET_RADIUS,
                    subdivisions: PLANET_SUBDIVISIONS,
                })),
                material: materials.add(Color::WHITE.into()),
                transform: Transform::from_xyz(x, y, z),
                ..Default::default()
            })
            .insert(PlanetComp)
            .id();

        let font = asset_server.load(FONT_PATH);

        let label_id = commands
            .spawn_bundle(TextBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                text: Text::with_section(
                    planet_name,
                    TextStyle {
                        font,
                        font_size: FONT_SIZE_DEFAULT,
                        color: FONT_COLOR,
                    },
                    TextAlignment::default(),
                ),
                global_transform: GlobalTransform {
                    translation: Vec3::new(0., 0., 0.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(PlanetLabel)
            .id();

        // println!("[{}] ({:.1},{:.1},{:.1})", planet_name, x, y, z);

        mapping.label_to_planet.insert(label_id, planet_id);
    }
}
