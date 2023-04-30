use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Error;
use std::path::Path;
use std::usize;
use num::{Bounded, NumCast, Zero};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use crate::adjacency::AdjacencyGenerator;

use crate::assigner::{MapColoringAssigner, MapColoringJob};
use crate::map2d::Map2D;
use crate::map2dnode::Map2DNode;
use crate::mapgen_presets;
use crate::position::{MapPosition, PositionKey};
use crate::sampler::{DistributionKey, MultinomialDistribution};
use crate::visualizers::{MapColor, MapVisualizer, RilPixelVisualizer};


#[derive(Serialize, Deserialize)]
pub struct GeneratorRuleset<A: DistributionKey> {
    layout_rules: MapColoringAssigner<A>,
    coloring_rules: HashMap<A, MapColor>,
    pub(crate) map_size: u32,
    pub(crate) adjacency: Option<String>,
    comments: Option<String>
}

impl<A: DistributionKey> GeneratorRuleset<A> {
    pub fn new(
        layout: MapColoringAssigner<A>,
        coloring: HashMap<A, MapColor>,
        map_size: Option<u32>,
        adjacency: Option<String>,
    ) -> Self {

        Self {
            layout_rules: layout,
            coloring_rules: coloring,
            map_size: map_size.unwrap_or(60u32),
            adjacency,
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

impl<'a, 'b, 'c, BS: Borrow<&'c str>> From<(&'a str, &'b str, Option<u32>, Option<BS>)> for GeneratorRuleset<i8> {
    fn from(value: (&'a str, &'b str, Option<u32>, Option<BS>)) -> Self {
        let colormap_path = value.0;
        let ruleset_path = value.1;
        let map_size = value.2;
        let adjacency = value.3.map(|a| a.borrow().trim().to_lowercase());

        let raw_colormap: HashMap<i8, ril::Rgb> = mapgen_presets::read_colormap(colormap_path);
        let colormap = raw_colormap.iter().map(
            |(k, v)| {
                let val = MapColor::from(v);
                (k.to_owned(), val)
            }
        ).collect();

        let ruleset = mapgen_presets::read_rules(ruleset_path);

        Self::new(ruleset, colormap, map_size, adjacency)
    }
}

impl<'a, 'b> From<(&'a str, &'b str, Option<u32>)> for GeneratorRuleset<i8> {
    fn from(value: (&'a str, &'b str, Option<u32>)) -> Self {
        Self::from((value.0, value.1, value.2, None::<&str>))
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

impl<DK: DistributionKey> GeneratorRuleset<DK> {
    pub fn generate_with_visualizer<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self, visualiser: V)
        where MP::Key: PositionKey + NumCast
    {
        let map_usize = <usize as NumCast>::from(self.map_size).unwrap();
        let map_size = <<MP as MapPosition<2>>::Key as NumCast>::from(self.map_size).unwrap_or(MP::Key::max_value());
        let assignment_rules = self.layout_rules.to_owned();
        let colormap: HashMap<DK, ril::Rgb> = self.coloring_rules.iter().map(
            |(k, v)| (
                k.to_owned(),
                v.to_owned().into()
            )
        ).collect();

        let mut tile_positions: SmallVec<[(<MP as MapPosition<2>>::Key, <MP as MapPosition<2>>::Key); 256]> = smallvec::SmallVec::with_capacity(map_usize * map_usize);
        for pos_x in num::range(MP::Key::zero(), map_size) {
            for pos_y in num::range(MP::Key::zero(), map_size) {
                tile_positions.push((pos_x, pos_y))
            }
        }

        let tile_iter: std::slice::Iter<'_, (<MP as MapPosition<2>>::Key, <MP as MapPosition<2>>::Key)> = tile_positions.iter();

        let test_tiles = tile_iter.map(
            |(x, y)| Map2DNode::with_possibilities(
                MP::from_dims([
                    <<MP as MapPosition<2>>::Key as NumCast>::from(x.to_owned()).unwrap(),
                    <<MP as MapPosition<2>>::Key as NumCast>::from(y.to_owned()).unwrap()
                ]),
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

    pub fn generate<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>>(&self)
        where MP::Key: PositionKey + NumCast + Into<u32>
    {
        let visualizer = RilPixelVisualizer::from(self.coloring_rules.to_owned());
        self.generate_with_visualizer::<AG, MP, RilPixelVisualizer<DK>>(visualizer)
    }
}


impl<DK: DistributionKey + Send + Sync> GeneratorRuleset<DK> {
    pub fn par_generate_with_visualizer<AG, MP, V>(&self, visualiser: V) where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Send + Sync,
        V: MapVisualizer<AG, DK, MP>,
    {
        let map_usize = <usize as NumCast>::from(self.map_size).unwrap();
        let map_size = <<MP as MapPosition<2>>::Key as NumCast>::from(self.map_size).unwrap_or(MP::Key::max_value());
        let assignment_rules = self.layout_rules.to_owned();
        let colormap: HashMap<DK, ril::Rgb> = self.coloring_rules.par_iter().map(
            |(k, v)| (
                k.to_owned(),
                v.to_owned().into()
            )
        ).collect();

        let mut tile_positions: SmallVec<[(<MP as MapPosition<2>>::Key, <MP as MapPosition<2>>::Key); 256]> = smallvec::SmallVec::with_capacity(map_usize * map_usize);
        for pos_x in num::range(MP::Key::zero(), map_size) {
            for pos_y in num::range(MP::Key::zero(), map_size) {
                tile_positions.push((pos_x, pos_y))
            }
        }

        let test_tiles = tile_positions.iter().map(
            |(x, y)| Map2DNode::with_possibilities(
                MP::from_dims([
                    <<MP as MapPosition<2>>::Key as NumCast>::from(x.to_owned()).unwrap(),
                    <<MP as MapPosition<2>>::Key as NumCast>::from(y.to_owned()).unwrap()
                ]),
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

    pub fn par_generate<AG, MP>(&self) where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync
    {
        let visualizer = RilPixelVisualizer::from(self.coloring_rules.to_owned());
        self.par_generate_with_visualizer::<AG, MP, RilPixelVisualizer<DK>>(visualizer)
    }
}
