use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Error;
use std::ops::{Div, Mul};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::usize;
use itertools::Itertools;
use num::{Bounded, NumCast, range_inclusive, Zero};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use crate::adjacency::AdjacencyGenerator;

use crate::assigner::{MapColoringAssigner, MapColoringJob};
use crate::map2d::Map2D;
use crate::map2dnode::{Map2DNode, MapNodeState};
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

impl<A: DistributionKey> From<GeneratorRuleset<A>> for MapColoringAssigner<A> {
    fn from(val: GeneratorRuleset<A>) -> MapColoringAssigner<A> {
        val.layout_rules
    }
}

impl<A: DistributionKey> From<GeneratorRuleset<A>> for HashMap<A, MapColor> {
    fn from(val: GeneratorRuleset<A>) -> Self {
        val.coloring_rules
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

impl<K: DistributionKey + Serialize> GeneratorRuleset<K> {
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
                Err(e)
            }
        }
    }
}

impl<DK: DistributionKey> GeneratorRuleset<DK> {
    pub fn default_map_builder<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self, map_size: u32) -> Map2D<AG, DK, MP>
        where MP::Key: PositionKey + NumCast
    {
        let raw_map_size = map_size;
        let map_usize = <usize as NumCast>::from(raw_map_size).unwrap();
        let true_map_size = <<MP as MapPosition<2>>::Key as NumCast>::from(raw_map_size).unwrap_or(MP::Key::max_value());

        let colormap: HashMap<DK, ril::Rgb> = self.coloring_rules.iter().map(
            |(k, v)| (
                k.to_owned(),
                v.to_owned().into()
            )
        ).collect();

        let mut tile_positions: SmallVec<[(<MP as MapPosition<2>>::Key, <MP as MapPosition<2>>::Key); 16384]> = smallvec::SmallVec::with_capacity(map_usize * map_usize);
        for pos_x in num::range(MP::Key::zero(), true_map_size) {
            for pos_y in num::range(MP::Key::zero(), true_map_size) {
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
                    colormap.keys().map(|k| k.to_owned())
                )
            )
        );
        Map2D::from_tiles(test_tiles)
    }

    /// Generates an empty (i.e. 'un-collapsed') map.
    /// This can be passed to a generate/infill function to collapse the map to a generated state.
    ///
    ///  **Arguments**:
    /// * `map_size` - optional u32, size of the map to generate (currently only square maps supported,
    /// so this corresponds to both X & Y axes. If None, falls back to the Ruleset value.
    /// * `map_builder` - optional function that takes a mapsize and returns a map.
    /// If None, uses the default method (allocate mapsize ** 2 unassigned tiles with uniform probability.)
    ///
    ///  **Returns**: a new Map2D.
    ///
    pub fn build_unassigned_map_with_size<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self, map_size: Option<u32>, map_builder: Option<Box<dyn Fn(u32) -> Map2D<AG, DK, MP>>>) -> Map2D<AG, DK, MP> where
        MP::Key: PositionKey + NumCast,
    {
        let raw_map_size = map_size.unwrap_or(self.map_size);
        match map_builder {
            Some(builder_fn) => builder_fn(raw_map_size),
            None => self.default_map_builder::<AG, MP, V>(raw_map_size)
        }
    }

    /// Generates an empty (i.e. 'uncollapsed') map using the default algorithm.
    /// This can be passed to a generate/infill function to collapse the map to a generated state.
    ///
    /// Takes no arguments, effectively a sugar for `self.build_unassigned_map_with_size(None, None)`
    ///
    /// **Returns**: a new Map2D.
    ///
    pub fn build_unassigned_map<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self) -> Map2D<AG, DK, MP>
        where MP::Key: PositionKey + NumCast
    {
        self.build_unassigned_map_with_size::<AG, MP, V>(None, None)
    }

    /// Creates a filled (i.e. 'collapsed') map,
    /// either from a partially un-collapsed map or entirely from scratch,
    /// and renders it using a provided MapVisualizer.
    ///
    ///  **Arguments**:
    /// * init_map - optional; a pre-initialized map to fill out.
    /// If None, will create a new map using the provided Ruleset's rules
    /// * visualizer - MapVisualizer interface to use to render the generated map.
    ///
    /// **Returns**: nothing (i.e. Unit); a render will be created as a side-effect.
    ///
    pub fn generate_with_visualizer<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self, init_map: Option<Map2D<AG, DK, MP>>, visualiser: V)
        where MP::Key: PositionKey + NumCast
    {
        let assignment_rules = self.layout_rules.to_owned();
        let gen_map = init_map.unwrap_or_else(
            || self.build_unassigned_map::<AG, MP, V>()
        );

        let mut job = MapColoringJob::new_with_queue(assignment_rules, gen_map);
        let map_result = job.queue_and_assign();
        let map_reader = map_result.read().unwrap();

        visualiser.visualise(&map_reader, None);
    }

    /// Creates a filled (i.e. 'collapsed') map,
    /// either from a partially un-collapsed map or entirely from scratch,
    /// and renders it using the *default* MapVisualizer.
    ///
    /// Effectively sugar over `self.generate_with_visualizer(init_map, <default visualizer>)`
    ///
    ///  **Arguments**:
    /// * init_map - optional; a pre-initialized map to fill out.
    /// If None, will create a new map using the provided Ruleset's rules
    ///
    /// **Returns**: nothing (i.e. Unit); a render will be created as a side-effect.
    ///
    pub fn generate_map<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>>(&self, init_map: Option<Map2D<AG, DK, MP>>)
        where MP::Key: PositionKey + NumCast + Into<u32>
    {
        let visualizer = RilPixelVisualizer::from(self.coloring_rules.to_owned());
        self.generate_with_visualizer::<AG, MP, RilPixelVisualizer<DK>>(init_map, visualizer)
    }

    /// Creates a filled (i.e. 'collapsed') map from scratch,
    /// and renders it using the *default* MapVisualizer.
    ///
    /// Effectively sugar over `self.generate_map(None, <default visualizer>)`
    ///
    /// **Arguments** - none
    ///
    /// **Returns**: nothing (i.e. Unit); a render will be created as a side-effect.
    ///
    pub fn generate<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>>(&self)
        where MP::Key: PositionKey + NumCast + Into<u32>
    {
        self.generate_map::<AG, MP>(None)
    }
}


impl<DK: DistributionKey + Send + Sync> GeneratorRuleset<DK> {
    pub fn default_map_builder_par<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self, map_size: u32) -> Map2D<AG, DK, MP>
        where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync,
        V: MapVisualizer<AG, DK, MP>,
    {
        let raw_map_size = map_size;
        let map_usize = <usize as NumCast>::from(raw_map_size).unwrap();
        let true_map_size = <<MP as MapPosition<2>>::Key as NumCast>::from(raw_map_size).unwrap_or(MP::Key::max_value());

        let colormap: HashMap<DK, ril::Rgb> = self.coloring_rules.par_iter().map(
            |(k, v)| (
                k.to_owned(),
                v.to_owned().into()
            )
        ).collect();

        let mut tile_positions: SmallVec<[(<MP as MapPosition<2>>::Key, <MP as MapPosition<2>>::Key); 16384]> = smallvec::SmallVec::with_capacity(map_usize * map_usize);
        for pos_x in num::range(MP::Key::zero(), true_map_size) {
            for pos_y in num::range(MP::Key::zero(), true_map_size) {
                tile_positions.push((pos_x, pos_y))
            }
        }

        let tile_iter = tile_positions.par_iter();

        let test_tiles: Vec<Map2DNode<AG, DK, MP>> = tile_iter.map(
            |(x, y)| Map2DNode::with_possibilities(
                MP::from_dims([
                    <<MP as MapPosition<2>>::Key as NumCast>::from(x.to_owned()).unwrap(),
                    <<MP as MapPosition<2>>::Key as NumCast>::from(y.to_owned()).unwrap()
                ]),
                MultinomialDistribution::uniform_over(
                    colormap.keys().map(|k| k.to_owned())
                )
            )
        ).collect();
        Map2D::from_tiles(test_tiles)
    }

    /// Generates an empty (i.e. 'un-collapsed') map.
    /// This can be passed to a generate/infill function to collapse the map to a generated state.
    ///
    ///  **Arguments**:
    /// * `map_size` - optional u32, size of the map to generate (currently only square maps supported,
    /// so this corresponds to both X & Y axes. If None, falls back to the Ruleset value.
    /// * `map_builder` - optional function that takes a mapsize and returns a map.
    /// If None, uses the default method (allocate mapsize ** 2 unassigned tiles with uniform probability.)
    ///
    ///  **Returns**: a new Map2D.
    ///
    pub fn build_unassigned_map_with_size_par<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self, map_size: Option<u32>, map_builder: Option<Box<dyn Fn(u32) -> Map2D<AG, DK, MP>>>) -> Map2D<AG, DK, MP> where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync,
        V: MapVisualizer<AG, DK, MP>,
    {
        let raw_map_size = map_size.unwrap_or(self.map_size);
        match map_builder {
            Some(builder_fn) => builder_fn(raw_map_size),
            None => self.default_map_builder_par::<AG, MP, V>(raw_map_size)
        }
    }

    /// Generates an empty (i.e. 'uncollapsed') map using the default algorithm.
    /// This can be passed to a generate/infill function to collapse the map to a generated state.
    ///
    /// Takes no arguments, effectively a sugar for `self.build_unassigned_map_with_size(None, None)`
    ///
    /// **Returns**: a new Map2D.
    ///
    pub fn build_unassigned_map_par<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self) -> Map2D<AG, DK, MP> where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync,
        V: MapVisualizer<AG, DK, MP>,
    {
        self.build_unassigned_map_with_size_par::<AG, MP, V>(None, None)
    }

    /// Creates a filled (i.e. 'collapsed') map,
    /// either from a partially un-collapsed map or entirely from scratch,
    /// and renders it using a provided MapVisualizer.
    ///
    ///  **Arguments**:
    /// * init_map - optional; a pre-initialized map to fill out.
    /// If None, will create a new map using the provided Ruleset's rules
    /// * visualizer - MapVisualizer interface to use to render the generated map.
    ///
    /// **Returns**: nothing (i.e. Unit); a render will be created as a side-effect.
    ///
    pub fn generate_with_visualizer_par<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self, init_map: Option<Map2D<AG, DK, MP>>, visualiser: V) where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync,
        V: MapVisualizer<AG, DK, MP>,
    {
        let assignment_rules = self.layout_rules.to_owned();
        let gen_map = init_map.unwrap_or_else(
            || self.build_unassigned_map_par::<AG, MP, V>()
        );

        let mut job = MapColoringJob::new_with_queue(assignment_rules, gen_map);
        let map_result = job.queue_and_assign();
        let map_reader = map_result.read().unwrap();

        visualiser.visualise(&map_reader, None);
    }

    pub fn regenerate_region<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>
    >(
        &self,
        src_map: &Map2D<AG, DK, MP>,
        start_pos: [MP::Key; 2],
        end_pos: [MP::Key; 2]
    ) -> Arc<RwLock<Map2D<AG, DK, MP>>> where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync
    {
        let mut newmap = src_map.to_owned();

        let min_pos_dimval = start_pos.first().unwrap().to_owned();
        let max_pos_dimval = end_pos.first().unwrap().to_owned();

        let tilerange = range_inclusive(
            min_pos_dimval,
            max_pos_dimval
        );
        let fuserange = tilerange.to_owned().cartesian_product(tilerange);

        let targ_tiles = fuserange.filter_map(
            |(x, y)| {
                let dims = [x, y];
                let map_pos = MP::from_dims(dims);
                let map_tile = src_map.get(map_pos);
                map_tile
            }
        );

        let new_assignment_rules = self.layout_rules.to_owned();
        let trans_rules = new_assignment_rules.transition_rules.to_owned();
        let map_keys = trans_rules.into_keys();

        newmap.unassign_tiles(
            targ_tiles,
            MultinomialDistribution::uniform_over(
                map_keys
            )
        );

        let unity: MP::Key = num::NumCast::from(1).unwrap();

        let edge_tiles_x = range_inclusive(min_pos_dimval, max_pos_dimval).filter_map(
            |x| {
                let dims = [x, min_pos_dimval];
                let map_pos = MP::from_dims(dims);
                let map_tile = src_map.get(map_pos);
                map_tile
            }
        ).map(
            |tile| {
                let tile_reader = tile.read().unwrap();
                let pos = tile_reader.position.get_dims();
                let adj_pos = MP::from_dims([pos[0], pos[1] - unity]);
                (tile, adj_pos)
            }
        ).chain(
        range_inclusive(min_pos_dimval, max_pos_dimval).filter_map(
            |x| {
                let dims = [x, max_pos_dimval];
                let map_pos = MP::from_dims(dims);
                let map_tile = src_map.get(map_pos);
                map_tile
            }).map(|tile| {
                let tile_reader = tile.read().unwrap();
                let pos = tile_reader.position.get_dims();
                let adj_pos = MP::from_dims([pos[0], pos[1] + unity]);
                (tile, adj_pos)
            }
        ));

        let edge_tiles_y = range_inclusive(min_pos_dimval, max_pos_dimval).filter_map(
            |y| {
                let dims = [min_pos_dimval, y];
                let map_pos = MP::from_dims(dims);
                let map_tile = src_map.get(map_pos);
                map_tile
            }
        ).map(
            |tile| {
                let tile_reader = tile.read().unwrap();
                let pos = tile_reader.position.get_dims();
                let adj_pos = MP::from_dims([pos[0] - unity, pos[1]]);
                (tile, adj_pos)
            }
        ).chain(
        range_inclusive(min_pos_dimval, max_pos_dimval).filter_map(
            |y| {
                let dims = [max_pos_dimval, y];
                let map_pos = MP::from_dims(dims);
                let map_tile = src_map.get(map_pos);
                map_tile
            }).map(|tile| {
                let tile_reader = tile.read().unwrap();
                let pos = tile_reader.position.get_dims();
                let adj_pos = MP::from_dims([pos[0] + unity, pos[1]]);
                (tile, adj_pos)
            }
        ));

        edge_tiles_x
            .chain(edge_tiles_y)
            .for_each(|(tile, adj_pos)| {
                let mut tile_writer = tile.write().unwrap();
                let raw_assigned_neighbor = src_map.get(adj_pos);
                
                if let Some(assigned_neighbor) = raw_assigned_neighbor {
                    let neigh_reader = assigned_neighbor.read().unwrap();
                    let neigh_dist = match &neigh_reader.state {
                        MapNodeState::Undecided(distribution) => distribution.to_owned(),
                        MapNodeState::Finalized(assignment) => MultinomialDistribution::uniform_over(
                            Some(assignment.to_owned())
                        )
                    };
                    let neighbor_state = &tile_writer.state.to_owned();
                    let my_dist = match &neighbor_state {
                        MapNodeState::Undecided(distribution) => distribution,
                        MapNodeState::Finalized(_) => panic!("Tile should have been unassigned!")
                    };
                    let updated_dist = neigh_dist.joint_probability(my_dist);
                    tile_writer.state = MapNodeState::Undecided(updated_dist);
                }
        });

        let mut coloring = MapColoringJob::new_with_queue(new_assignment_rules, newmap);
        let newmap_result = coloring.queue_and_assign();

        newmap_result.to_owned()
    }

    /// Showcase of Modifying In Blocks approach - generates a map, then edits
    /// the top-left quadrant by restting it to an unassigned state and filling it in again.
    /// The approach here is consistent (i.e. doesn't violate constraints), but may exhibit
    /// directional artifacts on the quadrant edge (usually, unnaturally straight lines).
    pub fn generate_with_visualizer_par_mib<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>, V: MapVisualizer<AG, DK, MP>>(&self, init_map: Option<Map2D<AG, DK, MP>>, visualiser: V) where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync,
        V: MapVisualizer<AG, DK, MP>,
        V::Args: From<&'static str>
    {
        let assignment_rules = self.layout_rules.to_owned();
        let gen_map = init_map.unwrap_or_else(
            || self.build_unassigned_map_par::<AG, MP, V>()
        );

        let mut job = MapColoringJob::new_with_queue(assignment_rules, gen_map.to_owned());
        let map_result = job.queue_and_assign();
        let map_reader = map_result.read().unwrap();

        visualiser.visualise(&map_reader, None);

        let min_pos = map_reader.max_pos.get_dims().map(|d| d.div(num::NumCast::from(4).unwrap()));
        let max_pos = min_pos.map(|d| d.mul(num::NumCast::from(3).unwrap()));

        let newmap_result = self.regenerate_region::<AG, MP, V>(&gen_map, min_pos, max_pos);
        let newmap_reader = newmap_result.read().unwrap();
        visualiser.visualise(&newmap_reader, Some("editmap.png".into()));
    }

    /// Creates a filled (i.e. 'collapsed') map,
    /// either from a partially un-collapsed map or entirely from scratch,
    /// and renders it using the *default* MapVisualizer.
    ///
    /// Effectively sugar over `self.generate_with_visualizer(init_map, <default visualizer>)`
    ///
    ///  **Arguments**:
    /// * init_map - optional; a pre-initialized map to fill out.
    /// If None, will create a new map using the provided Ruleset's rules
    ///
    /// **Returns**: nothing (i.e. Unit); a render will be created as a side-effect.
    ///
    pub fn generate_map_par<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>>(&self, init_map: Option<Map2D<AG, DK, MP>>) where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync
    {
        let visualizer = RilPixelVisualizer::from(self.coloring_rules.to_owned());
        self.generate_with_visualizer_par_mib::<AG, MP, RilPixelVisualizer<DK>>(init_map, visualizer)
    }

    /// Creates a filled (i.e. 'collapsed') map from scratch,
    /// and renders it using the *default* MapVisualizer.
    ///
    /// Effectively sugar over `self.generate_map(None, <default visualizer>)`
    ///
    /// **Arguments** - none
    ///
    /// **Returns**: nothing (i.e. Unit); a render will be created as a side-effect.
    ///
    pub fn generate_par<AG: AdjacencyGenerator<2, Input = MP>, MP: MapPosition<2>>(&self) where
        AG: AdjacencyGenerator<2, Input = MP> + Send + Sync,
        MP: MapPosition<2> + Send + Sync,
        MP::Key: PositionKey + NumCast + Into<u32> + Send + Sync
    {
        self.generate_map_par::<AG, MP>(None)
    }
}
