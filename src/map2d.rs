use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
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

pub trait PositionKey: Copy + Clone + Add<Output = Self> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Zero + num::One + num::Bounded {}
impl<P: Copy + Clone + Add<Output = P> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Zero + num::One + num::Bounded> PositionKey for P {}


#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Position2D<P: PositionKey> {
    pub x: P,
    pub y: P
}

impl<P: PositionKey> Position2D<P> {
    pub fn new(x: P, y: P) -> Self {
        Self {x, y}
    }
}

impl<P: PositionKey> From<(P, P)> for Position2D<P> {
    fn from(value: (P, P)) -> Self {
        Self {
            x: value.0,
            y: value.1
        }
    }
}

impl<P: PositionKey> Into<(P, P)> for Position2D<P> {
    fn into(self) -> (P, P) {
        (self.x, self.y)
    }
}

impl<PA: PositionKey + Add<Output = PA>> Add for Position2D<PA> {
    type Output = Position2D<PA>;

    fn add(self, rhs: Self) -> Self::Output {
        Position2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}


impl<P: PositionKey> Position2D<P> {
    pub fn adjacents_cardinal(&self) -> ArrayVec<Position2D<P>, 8> {
        let mut adjacents: ArrayVec::<Position2D<P>, 8> = ArrayVec::new();
        let type_unity: P = num::one();
        let type_three: P = type_unity + type_unity + type_unity;

        for dim in 0..2 {

            let offset_range = num::range(
                 num::zero(),
                 type_three
            );

            for offset in offset_range {
                if offset == type_unity {
                    continue
                };
                let true_offset = offset - type_unity;

                let mut pos_buffer = [self.x, self.y];
                pos_buffer[dim] = pos_buffer[dim] + true_offset;

                let new_pos = Position2D {
                    x: pos_buffer[0].into(),
                    y: pos_buffer[1].into()
                };

                adjacents.push(new_pos);
            }
        }

        adjacents
    }

    pub fn adjacents_octile(&self) -> ArrayVec<Position2D<P>, 8> {
        let mut adjacents: ArrayVec<Position2D<P>, 8> = ArrayVec::new();

        let type_unity: P = P::one();
        let type_three = type_unity + type_unity + type_unity;

        let x_range = num::range(
            P::zero(),
            type_three
        );

        for raw_x_dim in x_range {
            let x_dim = raw_x_dim - type_unity;

            let y_range = num::range(
                P::zero(),
                type_three
            );

            for raw_y_dim in y_range {
                if raw_x_dim.is_one() && raw_y_dim.is_one() {
                    continue
                };
                let y_dim = raw_y_dim - type_unity;
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
pub struct Map2DNode<K: DistributionKey, P: PositionKey> {
    pub(crate) position: Position2D<P>,
    pub(crate) possibilities: MultinomialDistribution<K>,
    pub(crate) assignment: Option<K>
}

impl<K: DistributionKey, P: PositionKey> Map2DNode<K, P> {
    pub fn with_possibilities(position: Position2D<P>, possibilities: MultinomialDistribution<K>) -> Self<> {
        Self {
            position,
            possibilities,
            assignment: None
        }
    }

    pub fn with_assignment(position: Position2D<P>, assignment: K) -> Self<> {
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
pub enum MapNodeWrapper<K: DistributionKey, P: PositionKey> {
    Raw(Map2DNode<K, P>),
    Arc(Arc<RwLock<Map2DNode<K, P>>>)
}

impl<K: DistributionKey, P: PositionKey> MapNodeWrapper<K, P> {
    pub fn position(&self) -> Position2D<P> {
        match self { 
            Self::Raw(node) => node.position,
            Self::Arc(arc_node) => arc_node.read().unwrap().position
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MapNodeEntropyOrdering<K: DistributionKey, P: PositionKey> {
    pub node: MapNodeWrapper<K, P>
}

impl<K: DistributionKey, P: PositionKey> From<Map2DNode<K, P>> for MapNodeEntropyOrdering<K, P> {
    fn from(value: Map2DNode<K, P>) -> Self {
        Self {
            node: MapNodeWrapper::Raw(value.clone())
        }
    }
}

impl<K: DistributionKey, P: PositionKey> From<Arc<RwLock<Map2DNode<K, P>>>> for MapNodeEntropyOrdering<K, P> {
    fn from(value: Arc<RwLock<Map2DNode<K, P>>>) -> Self {
        Self {
            node: MapNodeWrapper::Arc(value.clone())
        }
    }
}

impl<K: DistributionKey, P: PositionKey> PartialEq<Self> for MapNodeEntropyOrdering<K, P> {
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

impl<K: DistributionKey, P: PositionKey> Eq for MapNodeEntropyOrdering<K, P> {}

impl<K: DistributionKey, P: PositionKey> PartialOrd for MapNodeEntropyOrdering<K, P> {
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

impl<K: DistributionKey, P: PositionKey> Ord for MapNodeEntropyOrdering<K, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub type ThreadsafeNodeRef<K, P> = Arc<RwLock<Map2DNode<K, P>>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Map2D<K: DistributionKey, P: PositionKey> {
    pub tiles: Vec<ThreadsafeNodeRef<K, P>>,
    position_index: HashMap<Position2D<P>, ThreadsafeNodeRef<K, P>>,
    pub undecided_tiles: HashMap<Position2D<P>, ThreadsafeNodeRef<K, P>>,
    pub(crate) min_pos: Position2D<P>,
    pub(crate) max_pos: Position2D<P>,
}

impl<K: DistributionKey, P: PositionKey> Map2D<K, P> {
    pub fn from_tiles<I: IntoIterator<Item=Map2DNode<K, P>>>(tiles: I) -> Map2D<K, P> {
        let iterator = tiles.into_iter();
        let size_estimate = iterator.size_hint().1.unwrap_or( iterator.size_hint().0);

        let mut tile_vec: Vec<ThreadsafeNodeRef<K, P>> = Vec::with_capacity(size_estimate);
        let mut position_hashmap: HashMap<Position2D<P>, ThreadsafeNodeRef<K, P>> = HashMap::with_capacity(size_estimate);
        let mut undecided_hashmap: HashMap<Position2D<P>, ThreadsafeNodeRef<K, P>> = HashMap::with_capacity(size_estimate);
        let mut minx = None;
        let mut miny = None;
        let mut maxx = None;
        let mut maxy = None;

        for tile in iterator {
            let tile_arc = Arc::new(RwLock::new(tile));
            let tile_arc_reader = tile_arc.read().unwrap();
            let tile_pos = tile_arc_reader.position;

            if tile_pos.x < minx.unwrap_or(P::max_value()) { minx = Some(tile_pos.x) };
            if tile_pos.y < miny.unwrap_or(P::max_value()) { miny = Some(tile_pos.y) };
            if tile_pos.x > maxx.unwrap_or(P::min_value()) { maxx = Some(tile_pos.x) };
            if tile_pos.y > maxy.unwrap_or(P::min_value()) { maxy = Some(tile_pos.y) };

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
                minx.unwrap_or(maxx.unwrap_or(P::zero())),
                miny.unwrap_or(maxy.unwrap_or(P::zero()))
            ),
            max_pos: Position2D::new(
                maxx.unwrap_or(minx.unwrap_or(P::zero())),
                maxy.unwrap_or(miny.unwrap_or(P::zero()))
            )
        }
    }

    pub fn adjacent_cardinal_from_pos(&self, pos: Position2D<P>) -> ArrayVec<ThreadsafeNodeRef<K, P>, 4> {
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

    pub fn adjacent_cardinal(&self, node: &Map2DNode<K, P>) -> ArrayVec<ThreadsafeNodeRef<K, P>, 4> {
        self.adjacent_cardinal_from_pos(node.position)
    }

    pub fn adjacent_octile_from_pos(&self, pos: Position2D<P>) -> ArrayVec<ThreadsafeNodeRef<K, P>, 8> {
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

    pub fn adjacent_octile(&self, node: &Map2DNode<K, P>) -> ArrayVec<ThreadsafeNodeRef<K, P>, 8> {
        self.adjacent_octile_from_pos(node.position)
    }

    pub fn get(&self, key: Position2D<P>) -> Option<&ThreadsafeNodeRef<K, P>> {
        self.position_index.get(&key)
    }

    pub fn finalize_tile<'n>(&'n mut self, tile: &'n ThreadsafeNodeRef<K, P>, assignment: K) -> Option<&ThreadsafeNodeRef<K, P>> {
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
