use std::{collections::HashMap, fmt::Debug};

use bevy::prelude::*;
use pathfinding::prelude::directions::W;
use rand::{thread_rng, Rng};

const FONT_SIZE_DEFAULT: f32 = 20.;
const RANDOM_SPACE_LIMIT: f32 = 6000.;
const PLANET_RADIUS: f32 = 50.;
const PLANET_SUBDIVISIONS: usize = 1;
const FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";
const FONT_COLOR: Color = Color::GOLD;
const LABEL_FADE_DISTANCE: f32 = 4000.;
const CONN_MAX_WIDTH: f32 = 20.;
const CONN_MIN_WIDTH: f32 = 0.2;

use crate::{
    camera::SceneCam,
    universe::{generate_universe_cartography, Galaxy, Planet},
    WinSize,
};

use bevy_render::camera::Camera;

// we need to query entities during a startup system
//  so we use this trick to call the startup system
//   at runtime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum PlanetConnInitState {
    Todo,
    Done,
}

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
    name_to_planet: HashMap<String, Entity>,
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        let (galaxies, planets) = generate_universe_cartography();
        app.insert_resource(CartographyRes { planets, galaxies })
            .insert_resource(Index {
                label_to_planet: HashMap::new(),
                name_to_planet: HashMap::new(),
            })
            .add_startup_system(setup_planets)
            .add_state(PlanetConnInitState::Todo)
            .add_system_set(
                SystemSet::on_enter(PlanetConnInitState::Todo)
                    .with_system(setup_planetary_connections),
            )
            .add_system(update_text_position)
            .add_system(update_text_visibility);
        // .add_system(update_text_scale); // <- Too laggy, need to optimize performance first
    }
}

fn update_text_position(
    windows: Res<Windows>,
    mut text_q: Query<(Entity, &mut Style, &mut Visibility), With<PlanetLabel>>,
    planet_q: Query<&GlobalTransform, With<PlanetComp>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<SceneCam>>,
    index: Res<Index>,
    win_size: Res<WinSize>,
) {
    for (camera, cam_transform) in camera_q.iter() {
        for (text_entity, mut style, mut visibility) in text_q.iter_mut() {
            // Use index to query planet by its text label for performance saving
            let planet_entity = index.label_to_planet.get(&text_entity).unwrap();
            let planet_transform = planet_q.get(*planet_entity).unwrap();

            if let Some(coords) =
                camera.world_to_screen(&windows, cam_transform, planet_transform.translation)
            {
                // if planet not in sight, then skip update
                if coords.y > win_size.h * 2.
                    || coords.y < -win_size.h * 2.
                    || coords.x > win_size.w * 2.
                    || coords.x < -win_size.w * 2.
                {
                    continue;
                }
                style.position.left = Val::Px(coords.x);
                style.position.bottom = Val::Px(coords.y);
            }
        }
    }
}

fn update_text_visibility(
    windows: Res<Windows>,
    mut text_q: Query<(Entity, &mut Text, &mut Visibility), With<PlanetLabel>>,
    planet_q: Query<&GlobalTransform, With<PlanetComp>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<SceneCam>>,
    index: Res<Index>,
    win_size: Res<WinSize>,
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

            if let Some(coords) =
                camera.world_to_screen(&windows, cam_transform, planet_transform.translation)
            {
                // if planet not in sight or over fade distance, hide
                if dist >= LABEL_FADE_DISTANCE
                    || (coords.y > win_size.h * 2.
                        || coords.y < -win_size.h * 2.
                        || coords.x > win_size.w * 2.
                        || coords.x < -win_size.w * 2.)
                {
                    text_visibility.is_visible = false;
                } else {
                    text_visibility.is_visible = true;
                }
            }
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

fn setup_planets(
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

        // let planet_conn_weights: i32 = planet.conns.iter().map(|conn| conn.count).sum();
        // let radius = (planet_conn_weights as f32 / 10000.)
        //     * (PLANET_MAX_RADIUS - PLANET_MIN_RADIUS)
        //     + PLANET_MIN_RADIUS;

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
        mapping
            .name_to_planet
            .insert(planet_name.to_string(), planet_id);
    }
}

fn setup_planetary_connections(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    cartography: Res<CartographyRes>,
    index: Res<Index>,
    planet_q: Query<&Transform, With<PlanetComp>>,
) {
    for (planet_a_name, planet_a) in &cartography.planets {
        let planet_a_entity = index.name_to_planet.get(planet_a_name).unwrap();
        let planet_a_transform = planet_q.get(*planet_a_entity).unwrap();

        // generate lines for each planet connection
        for conn in &planet_a.conns {
            let (_, planet_b_name) = &conn.planet_pairs;
            let planet_b_entity = index.name_to_planet.get(planet_b_name).unwrap();
            let planet_b_transform = planet_q.get(*planet_b_entity).unwrap();
            // calculate the direction between planet a and b
            let rot = planet_a_transform.translation - planet_b_transform.translation;

            // find the middle point between a and b
            let middle_vec = (planet_a_transform.translation + planet_b_transform.translation) / 2.;

            // find distance between a and b as the length of cube
            let dist = planet_a_transform
                .translation
                .distance(planet_b_transform.translation);

            // calculate connection width based on its weight
            let weight = conn.count;
            let width: f32 = match weight {
                0..=2 => CONN_MIN_WIDTH,
                1000.. => CONN_MAX_WIDTH,
                x => (CONN_MAX_WIDTH - CONN_MIN_WIDTH) * (x as f32 / 1000.) + CONN_MIN_WIDTH,
            };
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(width, dist, width))),
                material: materials.add(Color::WHITE.into()),
                transform: Transform {
                    translation: middle_vec,
                    rotation: Quat::from_rotation_arc(
                        Vec3::Y,
                        (planet_b_transform.translation - planet_a_transform.translation)
                            .normalize(),
                    ),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }
}
