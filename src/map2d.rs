use std::cmp::{max, min, Ordering};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};
use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use crate::map::{IsArrayVec, MapNode, MapPosition, PositionKey, TileMap};
use crate::map_node_state::MapNodeState;
use crate::sampler::MultinomialDistribution;


#[derive(Clone, Serialize, Deserialize)]
pub enum MapNodeWrapper<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> {
    Raw(MN),
    Arc(Arc<RwLock<MN>>),
    _Fake(PhantomData<&'a MN>)
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> MapNodeWrapper<'a, DIMS, MN> {
    pub fn position(&'a self) -> MN::Position {
        let readpos;

        let result = match self {
            Self::Raw(node) => node.get_position(),
            Self::Arc(arc_node) => {
                let arc_read = arc_node.read().unwrap();
                readpos = arc_read;
                let pos = readpos.get_position();
                pos
            },
            Self::_Fake(_) => panic!("This should never be reachable!")
        };
        result.to_owned()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThreadsafeNodeRef<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> {
    inner: Arc<RwLock<MN>>,
    phantom_lifetime: PhantomData<&'a MN>
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> AsRef<MN> for ThreadsafeNodeRef<'a, DIMS, MN> {
    fn as_ref(&self) -> &MN {
        let node_guard = self.read().unwrap();
        let noderef = node_guard.deref();
        noderef
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> ThreadsafeNodeRef<'a, DIMS, MN> {
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, MN>> {
        self.inner.read()
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, MN>> {
        self.inner.write()
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> PartialOrd for ThreadsafeNodeRef<'a, DIMS, MN> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let me = self.read().unwrap();
        let you = other.read().unwrap();
        let myval = me.deref();
        let youval = you.deref();
        myval.partial_cmp(youval)
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> From<MN> for ThreadsafeNodeRef<'a, DIMS, MN> {
    fn from(value: MN) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
            phantom_lifetime: PhantomData
        }
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> PartialEq for ThreadsafeNodeRef<'a, DIMS, MN> {
    fn eq(&self, other: &Self) -> bool
    {
        let me = self.read().unwrap();
        let you = other.read().unwrap();
        let mypos = me.get_position();
        let yourpos = you.get_position();
        let equal = mypos.eq(&yourpos);
        equal
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> Eq for ThreadsafeNodeRef<'a, DIMS, MN> {}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> Hash for ThreadsafeNodeRef<'a, DIMS, MN> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let me = self.read().unwrap();
        me.get_position().hash(state);
        // me.state.hash(state);
    }
}


// #[derive(Serialize, Deserialize, Clone)]
#[derive(Clone)]
pub struct Map2D<'a, MN: MapNode<'a, 2>> {
    pub tiles: Vec<ThreadsafeNodeRef<'a, 2, MN>>,
    position_index: HashMap<MN::Position, ThreadsafeNodeRef<'a, 2, MN>>,
    pub undecided_tiles: HashMap<MN::Position, ThreadsafeNodeRef<'a, 2, MN>>,
    pub(crate) min_pos: MN::Position,
    pub(crate) max_pos: MN::Position,
}


impl<'a, P: PositionKey, MN: MapNode<'a, 2, ReadAs=ThreadsafeNodeRef<'a, 2, MN>, PositionKey=P> + Clone> Map2D<'a, MN> {
    pub fn from_tiles<I: IntoIterator<Item=MN>>(tiles: I) -> Map2D<'a, MN>
    where
        <<<MN as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::DimArray as IntoIterator>::Item: PartialOrd<<<MN as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey>,
        P: From<<<MN as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey> + Into<<<MN as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey>
    {
        let iterator = tiles.into_iter();
        let size_estimate = iterator.size_hint().0;

        let mut tile_vec = Vec::with_capacity(size_estimate);
        let mut position_hashmap: HashMap<MN::Position, ThreadsafeNodeRef<'a, 2, MN>> = HashMap::with_capacity(size_estimate);
        let mut undecided_hashmap: HashMap<MN::Position, ThreadsafeNodeRef<'a, 2, MN>> = HashMap::with_capacity(size_estimate);

        let mut minx = None;
        let mut miny = None;
        let mut maxx = None;
        let mut maxy = None;

        for iter_item in iterator {
            let tile: MN = iter_item;
            let tile_pos = tile.get_position();
            let tile_pos_array = tile_pos.to_owned().to_dim_array();
            let pos_iter = tile_pos.to_owned().to_dim_array().concrete();

            let pos_x: P = pos_iter[0].into();
            let pos_y: P = pos_iter[1].into();

            let minpos: P = P::min_value();
            let maxpos: P = P::max_value();

            minx = Some(min(pos_x, minx.unwrap_or(minpos)));
            maxx = Some(max(pos_x, maxx.unwrap_or(maxpos)));
            miny = Some(min(pos_y, miny.unwrap_or(minpos)));
            maxy = Some(max(pos_y, maxy.unwrap_or(maxpos)));

            tile_vec.push(tile.to_owned());
            let tile_pos = MN::Position::from_dim_array(&tile_pos_array);
            position_hashmap.insert(
                tile_pos.clone().into(),
                ThreadsafeNodeRef::from(tile.to_owned())
            );

            if !tile.get_state().is_assigned() {
                undecided_hashmap.insert(
                    tile_pos.into(),
                    ThreadsafeNodeRef::from(tile.to_owned())
                );
            }
        }

        // minx.unwrap_or(maxx.unwrap_or(MN::Position::zero())),
        // miny.unwrap_or(maxy.unwrap_or(MN::Position::zero()))
        let safe_min_x = minx.unwrap_or(maxx.unwrap());
        let safe_min_y = miny.unwrap_or(maxy.unwrap());
        let safe_max_x = maxx.unwrap_or(minx.unwrap());
        let safe_max_y = maxy.unwrap_or(miny.unwrap());

        let min_pos_arr = <<MN as MapNode<'a, 2>>::Position as MapPosition<2, 8>>::DimArray::from_arr([
            safe_min_x.to_owned().into(),
            safe_min_y.to_owned().into()
        ]);
        let min_pos = MN::Position::from_dim_array(&min_pos_arr);

        let max_pos_arr = <<MN as MapNode<'a, 2>>::Position as MapPosition<2, 8>>::DimArray::from_arr([
            safe_max_x.to_owned().into(),
            safe_max_y.to_owned().into()
        ]);
        let max_pos = MN::Position::from_dim_array(&max_pos_arr);

        let tiles: Vec<MN> = tile_vec;

        Self::build(
            &tiles,
            position_hashmap,
            undecided_hashmap,
            min_pos,
            max_pos
        )
    }

    // pub fn adjacent_cardinal_from_pos(&self, pos: &MN::Position) -> ArrayVec<ThreadsafeNodeRef<'a, 2, MN>, 4> {
    //     pos
    //     .adjacent()
    //     .into_iter()
    //     .filter_map(
    //         |cand| {
    //             let k: <<MN as MapNode<'a, 2>>::Position as MapPosition<2, 4>>::Me = cand;
    //             self.position_index
    //                 .get(&cand)
    //                 .map(|x| x.to_owned())
    //         }
    //     ).collect()
    // }
    //
    // pub fn adjacent_cardinal(&self, node: &MN) -> ArrayVec<ThreadsafeNodeRef<'a, 2, MN>, 4> {
    //     self.adjacent_cardinal_from_pos(&node.get_position())
    // }

    // pub fn adjacent_octile_from_pos(&self, pos: &MN::Position) -> ArrayVec<ThreadsafeNodeRef<'a, 2, MN>, 8> {
    //     pos
    //     .adjacent()
    //     .concrete()
    //     .into_iter()
    //     .filter_map(
    //         |cand| {
    //             self.position_index
    //                 .get(&cand)
    //                 .map(|x| x.to_owned())
    //         }
    //     ).collect()
    // }
    //
    // pub fn adjacent_octile(&self, node: &MN) -> ArrayVec<MN, 8> {
    //     self.adjacent_octile_from_pos(node.get_position())
    // }

    pub fn get(&'a self, key: &MN::Position) -> Option<&'a ThreadsafeNodeRef<'a, 2, MN>> {
        let out = self.position_index.get(key);
        out
    }

    pub fn finalize_tile<'n>(&'n mut self, tile: &'n ThreadsafeNodeRef<'a, 2, MN>, assignment: MN::Assignment) -> Option<&ThreadsafeNodeRef<'a, 2, MN>> {
        let tile_writer = tile.inner.write();
        match tile_writer {
            Ok(writeable) => {
                let mut state = writeable.get_state();
                state = MapNodeState::finalized(assignment);
                let position = writeable.get_position();
                let removed = self.undecided_tiles.remove(&position);
                match removed {
                    Some(_) => Some(tile),
                    None => None
                }
            },
            Err(_) => None
        }
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> MapNode<'a, DIMS> for ThreadsafeNodeRef<'a, DIMS, MN>
{
    type PositionKey = <<ThreadsafeNodeRef<'a, DIMS, MN> as MapNode<'a, DIMS>>::Position as MapPosition<'a, DIMS, 8>>::PositionKey;
    type ReadAs = RwLockReadGuard<'a, MN>;
    type Position = MN::Position;
    type Assignment = MN::Assignment;

    fn with_possibilities(position: Self::Position, possibilities: MultinomialDistribution<Self::Assignment>) -> Self {
        let new_inner = MN::with_possibilities(position, possibilities);
        Self::from(new_inner)
    }

    fn read_node(&'a self) -> Self::ReadAs {
        let unwrapped = self.read().unwrap();
        let result = unwrapped;
        result
    }

    fn get_position(&'a self) -> Self::Position {
        let node = self.read_node();
        let pos = node.get_position();
        pos
    }

    fn get_state(&self) -> MapNodeState<Self::Assignment> {
        let node = self.read_node();
        node.get_state()
    }

    fn get_entropy(&self) -> f32 { self.read_node().get_entropy() }

    fn adjacent<I: IntoIterator<Item=Self::Position>>(&self) -> I {
        self.read_node().adjacent()
    }
}

impl<'a, MN: MapNode<'a, 2>> TileMap<'a, 2, MN> for Map2D<'a, MN> {
    type TileContainer = Vec<ThreadsafeNodeRef<'a, 2, MN>>;
    type AdjacentsArray = ArrayVec<ThreadsafeNodeRef<'a, 2, MN>, 8>;
    type PositionIndex = HashMap<MN::Position, ThreadsafeNodeRef<'a, 2, MN>>;
    type UndecidedIndex = HashMap<MN::Position, ThreadsafeNodeRef<'a, 2, MN>>;
    type NodeReadAs = &'a ThreadsafeNodeRef<'a, 2, MN>;

    fn parse_tiles<I: IntoIterator<Item=MN>>(tiles: &I) -> (Self::PositionIndex, Self::UndecidedIndex, MN::Position, MN::Position) {
        let pos_idx = Self::PositionIndex::new();
        let undecided_idx = Self::UndecidedIndex::new();
        let mut min_pos: Option<MN::Position> = None;
        let mut max_pos: Option<MN::Position> = None;

        let all_tiles: Vec<MN> = tiles.into_iter().collect();

        for tile in all_tiles.iter() {
            let tile_cast: &MN = tile;
            let tilepos = tile_cast.get_position();
            match min_pos {
                None => min_pos = Some(tilepos.to_owned()),
                Some(oldpos) => {
                    if tilepos < oldpos { min_pos = Some(tilepos.to_owned()) }
                }
            }
            match max_pos {
                None => max_pos = Some(tilepos.to_owned()),
                Some(oldpos) => {
                    if tilepos > oldpos { max_pos = Some(tilepos.to_owned()) }
                }
            }
        }

        (pos_idx, undecided_idx, min_pos.unwrap(), max_pos.unwrap())
    }

    fn build<I: IntoIterator<Item=MN>>(
        tiles: &I,
        position_index: Self::PositionIndex,
        undecided_tiles: Self::UndecidedIndex,
        min_pos: MN::Position,
        max_pos: MN::Position

    ) -> Self {
        Self {
            tiles: tiles.into_iter().map(|t| ThreadsafeNodeRef::from(t)).collect(),
            position_index,
            undecided_tiles,
            min_pos,
            max_pos
        }
    }

    fn adjacent(&self, position: &MN::Position) -> Self::AdjacentsArray {
        let valid_positions = position.adjacent().concrete().iter().filter_map(
            |cand| {
                self.position_index
                    .get(&cand)
                    .map(|x| x.to_owned())
            }
        );
        let result: Self::AdjacentsArray = valid_positions.collect();
        result
    }

    fn get_min_pos(&self) -> &MN::Position {
        &self.min_pos
    }

    fn get_max_pos(&self) -> &MN::Position {
        &self.max_pos
    }

    fn get_node_by_pos(&self, key: &MN::Position) -> Option<Self::NodeReadAs> {
        let fetched = self.position_index.get(key);
        fetched
    }

    fn get_tiles(&self) -> Self::TileContainer {
        self.tiles.to_vec()
    }

    fn get_unassigned(self) -> HashMap<MN::Position, ThreadsafeNodeRef<'a, 2, MN>> {
        self.undecided_tiles
    }

    fn read_access(&self, node: &MN) -> Self::NodeReadAs {
        let pos = node.get_position();
        let raw_read = self.position_index.get(&pos);
        raw_read.unwrap()
    }
}

