use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use arrayvec::ArrayVec;
use num::{Bounded, Zero};
use serde::{Deserialize, Serialize};
use crate::sampler::DistributionKey;
use crate::map2dnode::{Map2DNode, MapNodeState, ThreadsafeNodeRef};
use crate::position::{MapPosition};

#[derive(Serialize, Deserialize, Clone)]
pub struct Map2D<K: DistributionKey, MP: MapPosition<2>> {
    pub tiles: Vec<ThreadsafeNodeRef<K, MP>>,
    position_index: HashMap<MP, ThreadsafeNodeRef<K, MP>>,
    pub undecided_tiles: HashMap<MP, ThreadsafeNodeRef<K, MP>>,
    pub(crate) min_pos: MP,
    pub(crate) max_pos: MP,
}

impl<K: DistributionKey, MP: MapPosition<2>> Map2D<K, MP> {
    pub fn from_tiles<I: IntoIterator<Item=Map2DNode<K, MP>>>(tiles: I) -> Map2D<K, MP> {
        let iterator = tiles.into_iter();
        let size_estimate = iterator.size_hint().0;

        let mut tile_vec: Vec<ThreadsafeNodeRef<K, MP>> = Vec::with_capacity(size_estimate);
        let mut position_hashmap: HashMap<MP, ThreadsafeNodeRef<K, MP>> = HashMap::with_capacity(size_estimate);
        let mut undecided_hashmap: HashMap<MP, ThreadsafeNodeRef<K, MP>> = HashMap::with_capacity(size_estimate);
        let mut minx = None;
        let mut miny = None;
        let mut maxx = None;
        let mut maxy = None;

        for tile in iterator {
            let cast_tile: Map2DNode<K, MP> = tile;
            let tile_pos = cast_tile.position.get_dims();

            let tile_pos_x = tile_pos.get(0).unwrap().clone();
            let tile_pos_y = tile_pos.get(1).unwrap().clone();

            let tile_arc = Arc::new(RwLock::new(cast_tile));
            let tile_arc_reader = tile_arc.read().unwrap();
            let tile_pos = tile_arc_reader.position;

            if tile_pos_x < minx.unwrap_or(MP::Key::max_value()) { minx = Some(tile_pos_x)};
            if tile_pos_y < miny.unwrap_or(MP::Key::max_value()) { miny = Some(tile_pos_y)};
            if tile_pos_x > maxx.unwrap_or(MP::Key::min_value()) { maxx = Some(tile_pos_x)};
            if tile_pos_y > maxy.unwrap_or(MP::Key::min_value()) { maxy = Some(tile_pos_y)};

            tile_vec.push(tile_arc.to_owned());
            position_hashmap.insert(tile_pos, tile_arc.to_owned());

            if !tile_arc_reader.state.is_assigned() {
                undecided_hashmap.insert(tile_pos, tile_arc.to_owned());
            }
        }

        Self {
            tiles: tile_vec,
            position_index: position_hashmap,
            undecided_tiles: undecided_hashmap,
            min_pos:  MP::from_dims([
                minx.unwrap_or(maxx.unwrap_or(MP::Key::zero())),
                miny.unwrap_or(maxy.unwrap_or(MP::Key::zero()))
            ]),
            max_pos: MP::from_dims([
                maxx.unwrap_or(minx.unwrap_or(MP::Key::zero())),
                maxy.unwrap_or(miny.unwrap_or(MP::Key::zero()))
            ])
        }
    }

    pub fn adjacent_cardinal_from_pos(&self, pos: MP) -> ArrayVec<ThreadsafeNodeRef<K, MP>, 4> {
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

    pub fn adjacent_cardinal(&self, node: &Map2DNode<K, MP>) -> ArrayVec<ThreadsafeNodeRef<K, MP>, 4> {
        self.adjacent_cardinal_from_pos(node.position)
    }

    pub fn adjacent_octile_from_pos(&self, pos: MP) -> ArrayVec<ThreadsafeNodeRef<K, MP>, 8> {
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

    pub fn adjacent_octile(&self, node: &Map2DNode<K, MP>) -> ArrayVec<ThreadsafeNodeRef<K, MP>, 8> {
        self.adjacent_octile_from_pos(node.position)
    }

    pub fn get(&self, key: MP) -> Option<&ThreadsafeNodeRef<K, MP>> {
        self.position_index.get(&key)
    }

    pub fn finalize_tile<'n>(&'n mut self, tile: &'n ThreadsafeNodeRef<K, MP>, assignment: K) -> Option<&ThreadsafeNodeRef<K, MP>> {
        let tile_writer = tile.write();
        match tile_writer {
            Ok(mut writeable) => {
                writeable.state = MapNodeState::finalized(assignment);
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
