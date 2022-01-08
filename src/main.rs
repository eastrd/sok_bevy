#![allow(unused)]

use std::collections::{HashMap, HashSet};

use bevy::{asset::Asset, prelude::*};
use camera::CameraPlugin;
use data::{get_all_relations, MetaRelation};
use debug::DebugPlugin;
use scene::ScenePlugin;

mod camera;
mod data;
mod debug;
mod scene;

// Describe a single connection between two planets
#[derive(Debug)]
struct Connection {
    planet_pairs: HashSet<(String, String)>,
    count: i32,
}

// Describe a planet and its connections
struct Planet {
    name: String,
    conns: Vec<Connection>,
    // One planet can belong to multiple galaxies
    belong_galaxy: HashSet<String>,
}

// Describe the set (e.g. Stack Overflow, Ask Ubuntu) of a bunch of planets
struct Galaxy {
    name: String,
}

fn generate_universe_cartography() -> (HashMap<String, Galaxy>, HashMap<String, Planet>) {
    // Generate Galaxies
    let meta_relations = get_all_relations("datasets/");
    let mut galaxies: HashMap<String, Galaxy> = HashMap::new();

    for meta in meta_relations.iter() {
        galaxies.insert(
            meta.domain.clone(),
            Galaxy {
                name: meta.domain.clone(),
            },
        );
    }

    // Discover all available planets across each galaxies into a queue
    let mut planets: HashMap<String, Planet> = HashMap::new();
    let mut discovered_planets: HashSet<String> = HashSet::new();

    for meta in meta_relations.iter() {
        for (planet_name, connected_tags) in meta.relation_map.iter() {
            if !discovered_planets.contains(planet_name) {
                discovered_planets.insert(planet_name.clone());

                planets.insert(
                    planet_name.clone(),
                    Planet {
                        name: planet_name.to_string(),
                        conns: vec![],
                        belong_galaxy: HashSet::new(),
                    },
                );
                let mut p = planets.get_mut(planet_name).unwrap();
                p.belong_galaxy.insert(meta.domain.clone());
            }
        }
    }

    // As planetary connections are bi-directional,
    //   need to filter out explored planets
    let mut explored_planets: HashSet<String> = HashSet::new();

    for meta in meta_relations.iter() {
        // create connections from tags
        for (planet_name, connected_tags) in meta.relation_map.iter() {
            let mut p = planets.get_mut(planet_name).unwrap();
            for t in connected_tags.iter() {
                // check if target planet has already been explored and skip accordingly
                if let Some(_) = explored_planets.get(&t.name) {
                    continue;
                }
                let mut conn = Connection {
                    planet_pairs: HashSet::new(),
                    count: t.count,
                };
                conn.planet_pairs
                    .insert((planet_name.to_string(), t.name.clone()));
                p.conns.push(conn);
            }
            explored_planets.insert(planet_name.to_string());
        }
    }

    // for (_, p) in planets.iter() {
    //     println!("[PLANET] {}", p.name);
    //     println!("[HOME GALAXY] {:?}", p.belong_galaxy);
    //     println!("[CONNS] {:?}", p.conns);
    // }

    return (galaxies, planets);
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.7, 0.7, 1.)))
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

    // Generate an interconnected universe of stack exchange
}
