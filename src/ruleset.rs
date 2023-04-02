use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Error;
use std::path::Path;
use std::usize;
use num::{NumCast};
use serde::{Serialize, Deserialize};

use crate::assigner::{MapColoringAssigner, MapColoringJob};
use crate::map2d::{Map2D, Map2DNode, Position2D, PositionKey};
use crate::mapgen_presets;
use crate::sampler::{DistributionKey, MultinomialDistribution};
use crate::visualizers::{MapColor, MapVisualizer, RilPixelVisualizer};


#[derive(Serialize, Deserialize)]
pub struct GeneratorRuleset<A: DistributionKey> {
    layout_rules: MapColoringAssigner<A>,
    coloring_rules: HashMap<A, MapColor>,
    pub(crate) map_size: u32,
    comments: Option<String>
}

impl<A: DistributionKey> GeneratorRuleset<A> {
    pub fn new(
        layout: MapColoringAssigner<A>,
        coloring: HashMap<A, MapColor>,
        map_size: Option<u32>
    ) -> Self {

        Self {
            layout_rules: layout,
            coloring_rules: coloring,
            map_size: map_size.unwrap_or(60u32),
            comments: None
        }
    }

}

impl<A: DistributionKey> Into<MapColoringAssigner<A>> for GeneratorRuleset<A> {
    fn into(self) -> MapColoringAssigner<A> {
        self.layout_rules.to_owned()
    }
}

impl<A: DistributionKey> Into<HashMap<A, MapColor>> for GeneratorRuleset<A> {
    fn into(self) -> HashMap<A, MapColor> {
        self.coloring_rules.to_owned()
    }
}

impl<'a, 'b> From<(&'a str, &'b str, Option<u32>)> for GeneratorRuleset<i8> {
    fn from(value: (&'a str, &'b str, Option<u32>)) -> Self {
        let colormap_path = value.0;
        let ruleset_path = value.1;
        let map_size = value.2;

        let raw_colormap: HashMap<i8, ril::Rgb> = mapgen_presets::read_colormap(colormap_path);
        let colormap = raw_colormap.iter().map(
            |(k, v)| {
                let val = MapColor::from(v);
                (k.to_owned(), val)
            }
        ).collect();

        let ruleset = mapgen_presets::read_rules(ruleset_path);

        Self::new(ruleset, colormap, map_size)
    }
}

impl<'a, 'b> From<(&'a str, &'b str, u32)> for GeneratorRuleset<i8> {
    fn from(value: (&'a str, &'b str, u32)) -> Self {
        Self::from((value.0, value.1, Some(value.2)))
    }
}

impl<'a, 'b> From<(&'a str, &'b str)> for GeneratorRuleset<i8> {
    fn from(value: (&'a str, &'b str)) -> Self {
        Self::from((value.0, value.1, None))
    }
}

impl<'d, K: DistributionKey + Serialize> GeneratorRuleset<K> {
    pub fn save<P: AsRef<Path>>(&self, filepath: P) -> &Self {
        let savefile = File::create(filepath).unwrap();
        serde_json::to_writer_pretty(savefile, self).unwrap();
        self
    }
}

impl GeneratorRuleset<i8> {
    pub fn load<P: AsRef<Path> + Debug>(filepath: P) -> Result<Self, Error> {
        let savefile = File::open(&filepath);
        match savefile {
            Ok(file) => match serde_json::from_reader(file) {
                Ok(inst) => Ok(inst),
                Err(e) => {
                    eprintln!("Failed to parse GeneratorRuleset savefile {:?}.", filepath);
                    Err(Error::from(e))
                }
            },
            Err(e) => {
                eprintln!("Failed to open GeneratorRuleset savefile {:?}.", filepath);
                Err(Error::from(e))
            }
        }
    }
}

impl GeneratorRuleset<i8> {
    pub fn generate_with_visualizer<P: PositionKey + NumCast, V: MapVisualizer<i8, P>>(&self, visualiser: V) {
        let map_usize = <usize as NumCast>::from(self.map_size).unwrap();
        let map_size = P::from(self.map_size).unwrap_or(P::max_value());
        let assignment_rules = self.layout_rules.to_owned();
        let colormap: HashMap<i8, ril::Rgb> = self.coloring_rules.iter().map(
            |(k, v)| (
                k.to_owned(),
                v.to_owned().into()
            )
        ).collect();

        let mut tile_positions = Vec::with_capacity(map_usize * map_usize);
        for pos_x in num::range(P::zero(), map_size) {
            for pos_y in num::range(P::zero(), map_size) {
                tile_positions.push((pos_x, pos_y))
            }
        }
        let test_tiles = tile_positions.iter().map(
            |(x, y)| Map2DNode::with_possibilities(
                Position2D::new(
                    P::from(x.to_owned()).unwrap(),
                    P::from(y.to_owned()).unwrap()
                ),
                MultinomialDistribution::uniform_over(
                    colormap.keys().into_iter().map(|k| k.to_owned())
                )
            )
        );
        let testmap = Map2D::from_tiles(test_tiles);

        let mut job = MapColoringJob::new_with_queue(assignment_rules, testmap);
        let map_result = job.queue_and_assign();
        let map_reader = &*map_result.read().unwrap();
        visualiser.visualise(map_reader);
    }

    pub fn generate<'a, P: PositionKey + NumCast + Into<u32>>(&self) {
        let visualizer = RilPixelVisualizer::from(self.coloring_rules.to_owned());
        self.generate_with_visualizer::<P, RilPixelVisualizer<i8>>(visualizer)
    }
}
