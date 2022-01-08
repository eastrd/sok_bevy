use pathfinding::prelude::dijkstra;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{
    collections::HashMap,
    fs::read_dir,
    io::{self, BufReader},
};

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct ConnectedTag {
    // 80% of original json with tag & count
    #[serde(rename(serialize = "t", deserialize = "t"))]
    pub name: String,
    #[serde(rename(serialize = "n", deserialize = "n"))]
    pub count: i32,
}

impl ConnectedTag {
    fn neighbours(
        &self,
        relation_map: HashMap<std::string::String, Vec<ConnectedTag>>,
    ) -> Vec<(ConnectedTag, i32)> {
        // Return a list of neighbours from current tag
        // If tag not in relation hashmap then return empty array
        match relation_map.get(&self.name) {
            Some(connected_tags) => connected_tags
                .iter()
                .map(|t| (t.clone(), (1000.0 / (t.count as f64)) as i32))
                .collect(),
            None => {
                vec![]
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetaRelation {
    // domain on Stack Exchange (e.g. StackOverflow, Unix)
    pub domain: String,
    // relation_map where it stores tag -> [tags+counters]
    pub relation_map: HashMap<String, Vec<ConnectedTag>>,
}

impl MetaRelation {
    pub fn new<'a>(domain: String, json_path: String) -> Result<MetaRelation, std::io::Error> {
        // Instantiate a new Meta Relation with given JSON file
        let file = File::open(json_path)?;
        let reader = BufReader::new(file);

        let mut data: HashMap<String, Vec<ConnectedTag>> = serde_json::from_reader(reader)?;
        // Sort all tags values during the read so we dont have to sort them again later
        data.iter_mut().for_each(|(_, tags)| {
            tags.sort_by(|a, b| {
                // Sort in descending order
                (&a.count).partial_cmp(&b.count).unwrap().reverse()
            })
        });

        Ok(MetaRelation {
            domain,
            relation_map: data,
        })
    }

    pub fn find_top_n(&self, tag_query: &str, n: usize) -> Vec<ConnectedTag> {
        if self.relation_map.contains_key(tag_query) {
            let tags = self.relation_map.get(tag_query).unwrap();
            // Have to return cloned copy here otherwise cannot refer to it with mutex in server module
            return tags
                .iter()
                .take(n)
                .map(|e| e.clone())
                .collect::<Vec<ConnectedTag>>();
        }
        vec![]
    }

    pub fn find_path(&self, start: String, goal: String) -> Option<(Vec<ConnectedTag>, i32)> {
        // Domain name already exists as this function being called from a meta relation instance
        // Start tag check is done in <ConnectTag>::neighbours()
        // Check if end tag exists,
        //  Otherwise the entire relation map will be unnecessary traversed
        match self.relation_map.get(&goal) {
            None => None,
            Some(_) => dijkstra(
                &ConnectedTag {
                    name: start,
                    count: 0,
                },
                |t| t.neighbours(self.relation_map.clone()),
                |t| *t.name == goal,
            ),
        }
    }
}

pub fn get_all_relations(relation_dir: &str) -> Vec<MetaRelation> {
    // Load all relation JSON files into memory
    let meta_relations: Vec<MetaRelation> = read_dir(relation_dir)
        .expect("error get relation dir")
        .map(|entry| {
            entry.map(|f| {
                (
                    f.path().to_string_lossy().to_string(),
                    f.file_name().to_string_lossy().to_string(),
                )
            })
        })
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap()
        .iter()
        .map(|(path, filename)| {
            // Get filename (exc .json) as domain name
            let domain = filename.split(".").next().unwrap().to_string();
            let json_path = path.to_string();
            println!("[READING] Domain: {} @ {}", domain, json_path);

            MetaRelation::new(domain, json_path).unwrap()
        })
        .collect();

    meta_relations
}
