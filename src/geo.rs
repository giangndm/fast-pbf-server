use std::{collections::HashMap, time::Instant};

use osmpbfreader::{OsmObj, Way};
use rstar::{
    primitives::{GeomWithData, Line},
    RTree,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WayInfo {
    way: Way,
    lines: Vec<[[f32; 2]; 2]>,
}

#[derive(Serialize, Deserialize)]
pub struct GeoIndex {
    tree: RTree<GeomWithData<Line<[f32; 2]>, (i64, i64)>>,
    ways: HashMap<i64, WayInfo>,
}

impl GeoIndex {
    pub fn new() -> GeoIndex {
        GeoIndex {
            tree: RTree::new(),
            ways: HashMap::new(),
        }
    }

    pub fn build(&mut self, path: &str) {
        let start = Instant::now();
        let mut pbf = osmpbfreader::OsmPbfReader::new(std::fs::File::open(path).unwrap());
        println!("Loaded pbf in {}ms", start.elapsed().as_millis());

        let mut nodes = HashMap::new();
        let mut nodes_count = 0;
        let mut ways_count = 0;
        let mut lines_count = 0;

        let tree = &mut self.tree;
        let ways = &mut self.ways;

        for obj in pbf.iter() {
            match obj {
                Ok(OsmObj::Node(node)) => {
                    nodes.insert(node.id.0, [node.lat() as f32, node.lon() as f32]);
                    nodes_count += 1;
                    if nodes_count % 10000 == 0 {
                        println!(
                            "Loaded {} ways {} nodes in {}ms",
                            ways_count,
                            nodes_count,
                            start.elapsed().as_millis()
                        );
                    }
                }
                Ok(OsmObj::Way(way)) => {
                    ways_count += 1;
                    if ways_count % 10000 == 0 {
                        println!(
                            "Loaded {} ways {} nodes in {}ms",
                            ways_count,
                            nodes_count,
                            start.elapsed().as_millis()
                        );
                    }
                    let name = if let Some(name) = way.tags.get("name") {
                        name.to_string()
                    } else {
                        continue;
                    };

                    let mut lines = vec![];
                    let mut start_point = None;
                    for node in &way.nodes {
                        if let Some(node_point) = nodes.get(&(node.0)) {
                            if let Some(start_point) = &mut start_point {
                                let line = Line::new(*start_point, *node_point);
                                lines_count += 1;
                                tree.insert(GeomWithData::new(line, (way.id.0, node.0)));
                                lines.push([*start_point, *node_point]);
                                *start_point = *node_point;
                            } else {
                                start_point = Some(*node_point);
                            }
                        }
                    }
                    ways.insert(way.id.0, WayInfo { way, lines });
                }
                _ => {}
            }
        }
        drop(nodes);
        drop(pbf);
        println!(
            "Loaded {} ways {} lines in {}ms",
            ways_count,
            lines_count,
            start.elapsed().as_millis()
        );
    }

    pub fn find(&self, lat: f32, lon: f32) -> Option<Vec<WayInfo>> {
        let mut way_ids = vec![];
        let mut ways = vec![];
        let lines = self.tree.nearest_neighbor_iter(&[lat, lon]);
        for line in lines {
            if !way_ids.contains(&line.data.0) {
                way_ids.push(line.data.0);
                if let Some(way) = self.ways.get(&line.data.0) {
                    ways.push(way.clone());
                    if ways.len() == 3 {
                        break;
                    }
                }
            }
        }
        Some(ways)
    }

    pub fn get_by_id(&self, id: i64) -> Option<WayInfo> {
        self.ways.get(&id).map(|way| way.clone())
    }
}
