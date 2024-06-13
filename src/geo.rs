use osmpbfreader::{NodeId, OsmObj};
use rstar::{
    primitives::{GeomWithData, Line},
    RTree,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};
use strum::EnumString;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GeometryType {
    Empty,
    Point,
    Line,
    Polygon,
    MultiPolygon,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LocationType {
    #[serde(rename = "way")]
    Way,
    #[serde(rename = "node")]
    Node,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LocationCategory {
    #[serde(rename = "highway")]
    Highway,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, EnumString, Debug)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum WayType {
    //highway
    Service,
    Cycleway,
    Path,
    Footway,
    Steps,
    Bridleway,
    MotorwayLink,
    PrimaryLink,
    TrunkLink,
    SecondaryLink,
    TertiaryLink,
    Residential,
    Track,
    Unclassified,
    Tertiary,
    Secondary,
    Primary,
    LivingStreet,
    Trunk,
    Motorway,
    Pedestrian,
    Road,
    Construction,

    //
    Unknown,
}

impl WayType {
    pub fn rank(&self) -> u8 {
        match &self {
            Self::Service => 27,
            Self::Cycleway => 27,
            Self::Path => 27,
            Self::Footway => 27,
            Self::Steps => 27,
            Self::Bridleway => 27,
            Self::MotorwayLink => 26,
            Self::PrimaryLink => 26,
            Self::TrunkLink => 26,
            Self::SecondaryLink => 26,
            Self::TertiaryLink => 26,
            Self::Residential => 26,
            Self::Track => 27,
            Self::Unclassified => 26,
            Self::Tertiary => 26,
            Self::Secondary => 26,
            Self::Primary => 26,
            Self::LivingStreet => 26,
            Self::Trunk => 26,
            Self::Motorway => 26,
            Self::Pedestrian => 26,
            Self::Road => 26,
            Self::Construction => 26,
            _ => 28,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocationExtraTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    oneway: Option<String>,
    #[serde(rename = "maxspeed", skip_serializing_if = "Option::is_none")]
    max_speed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lanes: Option<u8>,
    #[serde(rename = "turn:lanes", skip_serializing_if = "Option::is_none")]
    turn_lanes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeometryInfo {
    #[serde(rename = "type")]
    _type: GeometryType,
    coordinates: Vec<[f32; 2]>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddressInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    place_id: Option<u32>,
    osm_id: u32,
    osm_type: LocationType,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lon: Option<String>,
    category: LocationCategory,
    #[serde(rename = "type")]
    _type: WayType,
    #[serde(rename = "extratags")]
    extra_tags: LocationExtraTask,
    geometry: GeometryInfo,
}

#[derive(Serialize, Deserialize)]
pub struct GeoIndex {
    tree: RTree<GeomWithData<Line<[f32; 2]>, u32>>,
    ways: HashMap<u32, AddressInfo>,
}

impl GeoIndex {
    pub fn new() -> GeoIndex {
        GeoIndex {
            tree: RTree::new(),
            ways: HashMap::new(),
        }
    }

    pub fn build(&mut self, path: &str) {
        let mut missing_way_type: HashMap<String, ()> = HashMap::new();
        let start = Instant::now();
        let mut pbf = osmpbfreader::OsmPbfReader::new(std::fs::File::open(path).unwrap());
        println!("Loaded pbf in {}ms", start.elapsed().as_millis());

        let mut nodes = HashMap::new();
        let mut nodes_count = 0;
        let mut ways_count = 0;
        let mut lines_count = 0;
        let mut relations = 0;

        for obj in pbf.iter() {
            match obj {
                Ok(OsmObj::Node(node)) => {
                    nodes.insert(node.id.0, [node.lat() as f32, node.lon() as f32]);
                    nodes_count += 1;
                    if nodes_count % 100000 == 0 {
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
                    if ways_count % 100000 == 0 {
                        println!(
                            "Loaded {} ways {} nodes in {}ms",
                            ways_count,
                            nodes_count,
                            start.elapsed().as_millis()
                        );
                    }
                    let mut points = vec![];
                    let mut lines = vec![];
                    let mut start_point = None;
                    for node in &way.nodes {
                        if let Some(node_point) = nodes.get(&(node.0)) {
                            points.push(*node_point);
                            if let Some(start_point) = &mut start_point {
                                let line = Line::new(*start_point, *node_point);
                                lines_count += 1;
                                self.tree.insert(GeomWithData::new(line, way.id.0 as u32));
                                lines.push([*start_point, *node_point]);
                                *start_point = *node_point;
                            } else {
                                start_point = Some(*node_point);
                            }
                        }
                    }
                    let geometry = GeometryType::from(way.nodes.as_slice());
                    let (category, way_type) = if let Some(highway) = way.tags.get("highway") {
                        let way_type = match WayType::try_from(highway.as_str()) {
                            Ok(t) => t,
                            Err(_e) => {
                                if !missing_way_type.contains_key(highway.as_str()) {
                                    missing_way_type.insert(highway.to_string(), ());
                                    log::error!("unknown type highway {highway}");
                                }
                                WayType::Unknown
                            }
                        };
                        (LocationCategory::Highway, way_type)
                    } else {
                        log::debug!("missing data {:?}", way);
                        (LocationCategory::Unknown, WayType::Unknown)
                    };
                    let info = AddressInfo {
                        place_id: None,
                        osm_id: way.id.0 as u32,
                        osm_type: LocationType::Way,
                        name: way
                            .tags
                            .get("name")
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| "".to_string()),
                        lat: None,
                        lon: None,
                        category,
                        _type: way_type,
                        extra_tags: LocationExtraTask {
                            oneway: way.tags.get("oneway").map(|w| w.to_string()),
                            max_speed: way.tags.get("maxspeed").map(|w| w.to_string()),
                            lanes: way.tags.get("lanes").and_then(|w| w.as_str().parse().ok()),
                            turn_lanes: way.tags.get("turn:lanes").map(|w| w.to_string()),
                        },
                        geometry: GeometryInfo {
                            _type: geometry,
                            coordinates: points,
                        },
                    };
                    self.ways.insert(way.id.0 as u32, info);
                }
                Ok(OsmObj::Relation(_e)) => {
                    relations += 1;
                }
                _ => {}
            }
        }
        drop(nodes);
        drop(pbf);
        println!(
            "Loaded {} ways {} lines {} relations in {}ms",
            ways_count,
            lines_count,
            relations,
            start.elapsed().as_millis()
        );
    }

    pub fn find(&self, lat: f32, lon: f32, zoom: u8) -> Option<Vec<AddressInfo>> {
        let max_rank = match zoom {
            0 | 1 | 2 => 2,
            3 | 4 => 4,
            5 => 8,
            6 | 7 => 10,
            8 | 9 => 12,
            10 => 16,
            11 => 17,
            12 => 18,
            13 => 19,
            14 => 22,
            15 => 25,
            16 => 26,
            17 => 27,
            18 => 30,
            _ => 27,
        };

        let mut way_ids = vec![];
        let mut ways = vec![];
        let lines = self.tree.nearest_neighbor_iter(&[lat, lon]);
        for line in lines {
            if !way_ids.contains(&line.data) {
                way_ids.push(line.data);
                if let Some(way) = self.ways.get(&line.data) {
                    if !matches!(
                        way.geometry._type,
                        GeometryType::Polygon | GeometryType::MultiPolygon
                    ) && matches!(way.category, LocationCategory::Highway)
                        && way._type.rank() < max_rank
                    {
                        let point = line.geom().nearest_point(&[lat, lon]);
                        let mut way = way.clone();
                        way.lat = Some(point[0].to_string());
                        way.lon = Some(point[1].to_string());
                        if simple_distance([lat, lon], point) <= 0.006 {
                            ways.push(way);
                        } else {
                            break;
                        }
                        if ways.len() == 5 {
                            break;
                        }
                    }
                }
            }
        }
        Some(ways)
    }

    pub fn get_by_id(&self, id: i64) -> Option<AddressInfo> {
        self.ways.get(&(id as u32)).map(|way| way.clone())
    }
}

impl From<&[NodeId]> for GeometryType {
    fn from(nodes: &[NodeId]) -> Self {
        match nodes.len() {
            0 => Self::Empty,
            1 => Self::Point,
            2 => Self::Line,
            _ => {
                if nodes.first().unwrap() == nodes.last().unwrap() {
                    Self::Polygon
                } else {
                    Self::Line
                }
            }
        }
    }
}

fn simple_distance(p1: [f32; 2], p2: [f32; 2]) -> f32 {
    let dis = (p1[0] - p2[0]) * (p1[0] - p2[0]) + (p1[1] - p2[1]) * (p1[1] - p2[1]);
    dis.sqrt()
}
