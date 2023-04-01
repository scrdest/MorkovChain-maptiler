use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Add;
use std::sync::{Arc, RwLock};
use super::sampler::{MultinomialDistribution, DistributionKey};
use arrayvec::ArrayVec;
use serde::{Serialize, Deserialize};

// #[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
// enum PositionValue {
//     I16(i16, i16),
//     I32(i32, i32),
//     I64(i64, i64),
//     I128(i128, i128),
// }

#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Position2D {
    pub x: i64,
    pub y: i64
}

impl Position2D {
    pub fn new(x: i64, y: i64) -> Self {
        Self {x, y}
    }
}

impl From<(i64, i64)> for Position2D {
    fn from(value: (i64, i64)) -> Self {
        Self {
            x: value.0,
            y: value.1
        }
    }
}

impl Into<(i64, i64)> for Position2D {
    fn into(self) -> (i64, i64) {
        (self.x, self.y)
    }
}

impl Add for Position2D {
    type Output = Position2D;

    fn add(self, rhs: Self) -> Self::Output {
        Position2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Position2D {
    pub fn adjacents_cardinal(&self) -> ArrayVec<Position2D, 8> {
        let mut adjacents: ArrayVec::<Position2D, 8> = ArrayVec::new();

        for dim in 0..2 {
            for offset in -1..2 {
                if offset == 0 {
                    continue
                };

                let mut pos_buffer = [self.x, self.y];
                pos_buffer[dim] = pos_buffer[dim] + offset;

                let new_pos = Position2D {
                    x: pos_buffer[0],
                    y: pos_buffer[1]
                };

                adjacents.push(new_pos);
            }
        }

        adjacents
    }

    pub fn adjacents_octile(&self) ->Vec<Position2D> {
        let mut adjacents = Vec::with_capacity(8);

        for x_dim in -1..2 {
            for y_dim in -1..2 {
                if x_dim == 0 && y_dim == 0 {
                    continue
                };
                let new_pos = Position2D {
                    x: self.x + x_dim,
                    y: self.y + y_dim
                };
                adjacents.push(new_pos);
            }
        }

        adjacents
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map2DNode<N: DistributionKey> {
    pub(crate) position: Position2D,
    pub(crate) possibilities: MultinomialDistribution<N>,
    pub(crate) assignment: Option<N>
}

impl<N: DistributionKey> Map2DNode<N> {
    pub fn with_possibilities(position: Position2D, possibilities: MultinomialDistribution<N>) -> Self<> {
        Self {
            position,
            possibilities,
            assignment: None
        }
    }

    pub fn with_assignment(position: Position2D, assignment: N) -> Self<> {
        Self {
            position,
            possibilities: MultinomialDistribution::uniform_over(vec![]),
            assignment: Some(assignment)
        }
    }

    pub fn entropy(&self) -> f32 {
        match &self.assignment {
            Some(_) => f32::INFINITY,
            None => self.possibilities.entropy()
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MapNodeWrapper<K: DistributionKey> {
    Raw(Map2DNode<K>),
    Arc(Arc<RwLock<Map2DNode<K>>>)
}

impl<K: DistributionKey> MapNodeWrapper<K> {
    pub fn position(&self) -> Position2D {
        match self { 
            Self::Raw(node) => node.position,
            Self::Arc(arc_node) => arc_node.read().unwrap().position
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MapNodeEntropyOrdering<K: DistributionKey> {
    pub node: MapNodeWrapper<K>
}

impl<K: DistributionKey> From<Map2DNode<K>> for MapNodeEntropyOrdering<K> {
    fn from(value: Map2DNode<K>) -> Self {
        Self {
            node: MapNodeWrapper::Raw(value.clone())
        }
    }
}

impl<K: DistributionKey> From<Arc<RwLock<Map2DNode<K>>>> for MapNodeEntropyOrdering<K> {
    fn from(value: Arc<RwLock<Map2DNode<K>>>) -> Self {
        Self {
            node: MapNodeWrapper::Arc(value.clone())
        }
    }
}

impl<N: DistributionKey> PartialEq<Self> for MapNodeEntropyOrdering<N> {
    fn eq(&self, other: &Self) -> bool {
        let my_entropy = match &self.node {
            MapNodeWrapper::Raw(node_data) => node_data.entropy(),
            MapNodeWrapper::Arc(node_data) => node_data.read().unwrap().entropy(),
        };

        let other_entropy = match &other.node {
            MapNodeWrapper::Raw(node_data) => node_data.entropy(),
            MapNodeWrapper::Arc(node_data) => node_data.read().unwrap().entropy(),
        };

        my_entropy == other_entropy
    }
}

impl<N: DistributionKey> Eq for MapNodeEntropyOrdering<N> {}

impl<N: DistributionKey> PartialOrd for MapNodeEntropyOrdering<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let my_entropy = match &self.node {
            MapNodeWrapper::Raw(node_data) => node_data.entropy(),
            MapNodeWrapper::Arc(node_data) => node_data.read().unwrap().entropy(),
        };

        let other_entropy = match &other.node {
            MapNodeWrapper::Raw(node_data) => node_data.entropy(),
            MapNodeWrapper::Arc(node_data) => node_data.read().unwrap().entropy(),
        };

        match my_entropy == other_entropy {
            true => Some(Ordering::Equal),
            false => match my_entropy > other_entropy {
                true => Some(Ordering::Less),
                false => Some(Ordering::Greater)
            }
        }
    }
}

impl<N: DistributionKey> Ord for MapNodeEntropyOrdering<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub type ThreadsafeNodeRef<N> = Arc<RwLock<Map2DNode<N>>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Map2D<N: DistributionKey> {
    pub tiles: Vec<ThreadsafeNodeRef<N>>,
    position_index: HashMap<Position2D, ThreadsafeNodeRef<N>>,
    pub undecided_tiles: HashMap<Position2D, ThreadsafeNodeRef<N>>,
    pub(crate) min_pos: Position2D,
    pub(crate) max_pos: Position2D,
}

impl<N: DistributionKey> Map2D<N> {
    pub fn from_tiles<I: IntoIterator<Item=Map2DNode<N>>>(tiles: I) -> Map2D<N> {
        let iterator = tiles.into_iter();
        let size_estimate = iterator.size_hint().1.unwrap_or( iterator.size_hint().0);

        let mut tile_vec: Vec<ThreadsafeNodeRef<N>> = Vec::with_capacity(size_estimate);
        let mut position_hashmap: HashMap<Position2D, ThreadsafeNodeRef<N>> = HashMap::with_capacity(size_estimate);
        let mut undecided_hashmap: HashMap<Position2D, ThreadsafeNodeRef<N>> = HashMap::with_capacity(size_estimate);
        let mut minx = None;
        let mut miny = None;
        let mut maxx = None;
        let mut maxy = None;

        for tile in iterator {
            let tile_arc = Arc::new(RwLock::new(tile));
            let tile_arc_reader = tile_arc.read().unwrap();
            let tile_pos = tile_arc_reader.position;

            if tile_pos.x < minx.unwrap_or(i64::MAX) { minx = Some(tile_pos.x) };
            if tile_pos.y < miny.unwrap_or(i64::MAX) { miny = Some(tile_pos.y) };
            if tile_pos.x > maxx.unwrap_or(i64::MIN) { maxx = Some(tile_pos.x) };
            if tile_pos.y > maxy.unwrap_or(i64::MIN) { maxy = Some(tile_pos.y) };

            tile_vec.push(tile_arc.to_owned());
            position_hashmap.insert(tile_pos, tile_arc.to_owned());

            if tile_arc_reader.assignment.is_none() {
                undecided_hashmap.insert(tile_pos, tile_arc.to_owned());
            }
        }

        Self {
            tiles: tile_vec,
            position_index: position_hashmap,
            undecided_tiles: undecided_hashmap,
            min_pos: Position2D::new(
                minx.unwrap_or(maxx.unwrap_or(0)),
                miny.unwrap_or(maxy.unwrap_or(0))
            ),
            max_pos: Position2D::new(
                maxx.unwrap_or(minx.unwrap_or(0)),
                maxy.unwrap_or(miny.unwrap_or(0))
            )
        }
    }

    pub fn adjacent_cardinal_from_pos(&self, pos: Position2D) -> ArrayVec<ThreadsafeNodeRef<N>, 4> {
        pos
        .adjacents_cardinal()
        .into_iter()
        .filter_map(
            |cand| {
                self.position_index
                    .get(&cand)
                    .map(|x| x.to_owned())
            }
        ).collect()
    }

    pub fn adjacent_cardinal(&self, node: &Map2DNode<N>) -> ArrayVec<ThreadsafeNodeRef<N>, 4> {
        self.adjacent_cardinal_from_pos(node.position)
    }

    pub fn adjacent_octile_from_pos(&self, pos: Position2D) -> ArrayVec<ThreadsafeNodeRef<N>, 8> {
        pos
        .adjacents_octile()
        .into_iter()
        .filter_map(
            |cand| {
                self.position_index
                    .get(&cand)
                    .map(|x| x.to_owned())
            }
        ).collect()
    }

    pub fn adjacent_octile(&self, node: &Map2DNode<N>) -> ArrayVec<ThreadsafeNodeRef<N>, 8> {
        self.adjacent_octile_from_pos(node.position)
    }

    pub fn get(&self, key: Position2D) -> Option<&ThreadsafeNodeRef<N>> {
        self.position_index.get(&key)
    }

    pub fn finalize_tile<'n>(&'n mut self, tile: &'n ThreadsafeNodeRef<N>, assignment: N) -> Option<&ThreadsafeNodeRef<N>> {
        let tile_writer = tile.write();
        match tile_writer {
            Ok(mut writeable) => {
                writeable.assignment = Some(assignment);
                let removed = self.undecided_tiles.remove(&writeable.position);
                match removed {
                    Some(_) => Some(tile),
                    None => None
                }
            },
            Err(_) => None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_vector_addition_works_positives() {
        let pos_a = Position2D { x: 5, y: 3 };
        let pos_b = Position2D { x: 2, y: 6 };
        let result_pos = pos_a + pos_b;
        assert_eq!(result_pos.x, 7);
        assert_eq!(result_pos.y, 9);
    }

    #[test]
    fn position_vector_addition_works_one_negative() {
        let pos_a = Position2D { x: -5, y: -3 };
        let pos_b = Position2D { x: 2, y: 6 };
        let result_pos = pos_a + pos_b;
        assert_eq!(result_pos.x, -3);
        assert_eq!(result_pos.y, 3);
    }

    #[test]
    fn adjacents_cardinal_sane() {
        let pos = Position2D { x: 2, y: 6 };
        let results = pos.adjacents_cardinal();
        assert_eq!(results[0], Position2D { x: 1, y: 6 });
        assert_eq!(results[1], Position2D { x: 3, y: 6 });
        assert_eq!(results[2], Position2D { x: 2, y: 5 });
        assert_eq!(results[2], Position2D { x: 2, y: 7 });
    }

    #[test]
    fn serde_pos() {
        let pos = Position2D { x: 2, y: 6 };
        let results = serde_json::to_string(&pos).unwrap();
        assert!(results.len() > 0)
    }
}
