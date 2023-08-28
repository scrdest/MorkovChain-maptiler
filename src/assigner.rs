use std::borrow::Borrow;
use std::collections::binary_heap::BinaryHeap;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::{Rc};
use std::cell::{RefCell};

use serde::{Deserialize, Serialize};

use crate::map2d::Map2D;
use crate::sampler::{DistributionKey, MultinomialDistribution};
use crate::adjacency::AdjacencyGenerator;
use crate::map2dnode::{MapNodeEntropyOrdering, MapNodeState, MapNodeWrapper};
use crate::position::{MapPosition};

type Queue<AG, K, MP> = Rc<RefCell<BinaryHeap<MapNodeEntropyOrdering<AG, K, MP>>>>;


#[derive(Serialize, Deserialize)]
enum QueueState {
    Uninitialized,
    Initialized,
    // Processed,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MapColoringAssigner<K: DistributionKey> {
    pub(crate) transition_rules: HashMap<K, MultinomialDistribution<K>>,
    comments: Option<String>
}

impl<K: DistributionKey> MapColoringAssigner<K> {
    pub fn with_rules(rules: HashMap<K, MultinomialDistribution<K>>) -> Self {
        Self {
            transition_rules: rules,
            comments: None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MapColoringJob<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> {
    rules: MapColoringAssigner<K>,
    pub map: Rc<RefCell<Map2D<AG, K, MP>>>,
    queue: Queue<AG, K, MP>,
    queue_state: QueueState
}

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> MapColoringJob<AG, K, MP>
where <AG as AdjacencyGenerator<2>>::Input: Borrow<MP> + From<MP>
{
    pub fn new(rules: MapColoringAssigner<K>, map: Map2D<AG, K, MP>) -> Self {
        let wrapped_map = Rc::new(RefCell::new(map));

        let raw_queue = BinaryHeap::new();
        let wrapped_queue = Rc::new(RefCell::new(raw_queue));

        Self {
            rules,
            map: wrapped_map,
            queue: wrapped_queue,
            queue_state: QueueState::Uninitialized
        }
    }

    fn build_queue(&mut self) -> &Queue<AG, K, MP> {
        let map_reader = self.map.try_borrow().unwrap();
        let wrapped_queue = &self.queue;
        let mut queue_writer = wrapped_queue.try_borrow_mut().unwrap();

        for tile_lock in map_reader.undecided_tiles.values() {
            let tile_reader = tile_lock.try_borrow().unwrap();
            let is_assigned = tile_reader.state.is_assigned();
            if is_assigned { continue };

            let tile = tile_reader.deref().to_owned();
            let ord_tile = MapNodeEntropyOrdering::from(tile);

            queue_writer.push(ord_tile);
            break; // we only want the first uninitialized element
        }
        self.queue_state = QueueState::Initialized;
        wrapped_queue
    }

    pub fn new_with_queue(rules: MapColoringAssigner<K>, map: Map2D<AG, K, MP>) -> Self {
        let mut inst = Self::new(rules, map);
        inst.build_queue();
        inst
    }

    pub fn assign_map(&mut self) -> &Rc<RefCell<Map2D<AG, K, MP>>>
    {
        let mut queue_writer = self.queue.try_borrow_mut().unwrap();
        let map = &self.map;
        let mut map_operator = map.try_borrow_mut().unwrap();

        let mut enqueued = HashSet::with_capacity(map_operator.undecided_tiles.len());
        match queue_writer.peek() {
            Some(enqueued_node) => enqueued.insert(enqueued_node.node.position()),
            None => false
        };

        while !queue_writer.is_empty() {
            let assignee = &queue_writer.pop().unwrap().node;
            let raw_node = match assignee {
                MapNodeWrapper::Raw(node) => Rc::new(RefCell::new(node.to_owned())),
                MapNodeWrapper::Rc(node) => node.to_owned(),
            };
            let mut node = raw_node.try_borrow_mut().unwrap();
            let node_state = &node.state.to_owned();
            let curr_pos = node.position;

            enqueued.remove(&curr_pos);
            map_operator.undecided_tiles.remove(&curr_pos);

            let possibilities = match node_state {
                MapNodeState::Undecided(probas) => probas,
                MapNodeState::Finalized(_) => {
                    continue
                }
            };

            let new_assignment = possibilities.sample_with_default_rng();
            // println!("Assigning {:?} => {:?}", node.position, new_assignment);

            let self_rule_probas = match self.rules.transition_rules.get(&new_assignment) {
                Some(probas) => probas,
                None => continue
            };

            node.state = MapNodeState::from(new_assignment);

            let neighbors = map_operator.adjacent(node.deref());
            drop(node);

            neighbors.iter().for_each(|neighbor| {
                // println!("Acquiring lock for neighbor {:?}...", neighbor);
                let maybe_neighbor_rule_probas;
                {
                    // sub-scope to free up the reader after use
                    let neighbor_reader = neighbor.try_borrow().unwrap();
                    maybe_neighbor_rule_probas = match &neighbor_reader.state {
                        MapNodeState::Undecided(probas) => Some(probas.to_owned()),
                        MapNodeState::Finalized(_) => None
                    };
                }

                if let Some(neighbor_rule_probas) = maybe_neighbor_rule_probas {
                    let mut neighbor_writer = neighbor.try_borrow_mut().unwrap();
                    let new_possibilities = self_rule_probas.joint_probability(&neighbor_rule_probas);
                    neighbor_writer.state = MapNodeState::from(new_possibilities);
                    //println!("Assigned new probas for neighbor {:?}!", neighbor);

                    let neigh_pos = neighbor_writer.position;
                    drop(neighbor_writer);

                    let neigh_queued = enqueued.get(&neigh_pos).is_some();

                    if !neigh_queued {
                        let wrapped_neighbor = MapNodeEntropyOrdering::from(neighbor.to_owned());
                        queue_writer.push(wrapped_neighbor);
                    }

                }

                // no real return value
            })
        }

        map
    }

    pub fn queue_and_assign(&mut self) -> &Rc<RefCell<Map2D<AG, K, MP>>> {
        self.build_queue();
        self.assign_map()
    }
}


#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::map2dnode::Map2DNode;
    use crate::OctileAdjacencyGenerator;
    use crate::position2d::Position2D;
    use super::*;

    #[test]
    fn small_assignment() {
        const TEST_MAP_SIZE: i64 = 10;
        let tile_positions = (0..TEST_MAP_SIZE).cartesian_product(0..TEST_MAP_SIZE);
        let test_tiles = tile_positions.map(
            |(x, y)| Map2DNode::<
                OctileAdjacencyGenerator<Position2D<i64>>, i32, Position2D<i64>
            >::with_possibilities(
                Position2D::new(
                    i64::from(x),
                    i64::from(y)
                ),
                MultinomialDistribution::uniform_over(vec![1, 2, 3])
            )
        );
        let testmap = Map2D::from_tiles(test_tiles);
        let rules = HashMap::from([
            (1, MultinomialDistribution::from(
                HashMap::from([
                    (2, 1.),
                    (3, 5.)
                ])
            )),
            (2, MultinomialDistribution::from(
                HashMap::from([
                    (1, 5.),
                    (3, 1.)
                ])
            )),
            (3, MultinomialDistribution::from(
                HashMap::from([
                    (1, 1.),
                    (2, 5.)
                ])
            )),
        ]);

        let assignment_rules = MapColoringAssigner::with_rules(rules);
        let mut job = MapColoringJob::new_with_queue(assignment_rules, testmap);
        let pre_run_state = &job.map.try_borrow().unwrap().undecided_tiles.to_owned();
        assert!(pre_run_state.len() > 0);
        job.queue_and_assign();
        let post_run_state = &job.map.try_borrow().unwrap().undecided_tiles.to_owned();
        assert_eq!(post_run_state.len(), 0);
    }
}
