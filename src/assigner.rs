use std::collections::binary_heap::BinaryHeap;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use crate::map2d::{MapNodeWrapper, ThreadsafeNodeRef};
use crate::sampler::{DistributionKey, MultinomialDistribution};
use serde::{Deserialize, Serialize};
use crate::map::{MapNode, TileMap};
use crate::map_node_ordering::MapNodeEntropyOrdering;
use crate::map_node_state::MapNodeState;

type Queue<'a, const DIMS: usize, MN> = Arc<RwLock<BinaryHeap<MapNodeEntropyOrdering<'a, DIMS, MN>>>>;


#[derive(Serialize, Deserialize)]
enum QueueState {
    Uninitialized,
    Initialized,
    // Processed,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MapColoringAssigner<K: DistributionKey> {
    transition_rules: HashMap<K, MultinomialDistribution<K>>,
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

// #[derive(Serialize, Deserialize)]
#[derive(Serialize)]
pub struct MapColoringJob<'a, const DIMS: usize, MN: MapNode<'a, DIMS> + 'a, ARMN: AsRef<MN> + MapNode<'a, DIMS>, M: TileMap<'a, DIMS, MN>> {
    rules: MapColoringAssigner<MN::Assignment>,
    pub map: Arc<RwLock<M>>,
    queue: Queue<'a, DIMS, ARMN>,
    queue_state: QueueState,
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS> + Eq + Hash, M: TileMap<'a, DIMS, MN>> MapColoringJob<'a, DIMS, MN, ThreadsafeNodeRef<'a, DIMS, MN>, M> {
    pub fn new(rules: MapColoringAssigner<MN::Assignment>, map: M) -> Self {
        let wrapped_map = Arc::new(RwLock::new(map));

        let raw_queue = BinaryHeap::new();
        let wrapped_queue = Arc::new(RwLock::new(raw_queue));

        Self {
            rules,
            map: wrapped_map,
            queue: wrapped_queue,
            queue_state: QueueState::Uninitialized,
        }
    }

    fn build_queue(&'a mut self) -> &Queue<DIMS, ThreadsafeNodeRef<'a, DIMS, MN>> {
        let map_reader: RwLockReadGuard<M> = self.map.read().unwrap();
        let wrapped_queue = &self.queue;
        let mut queue_writer = wrapped_queue.write().unwrap();

        for raw_tile_lock in map_reader.get_unassigned().values() {
            let tile_lock: &ThreadsafeNodeRef<DIMS, MN> = raw_tile_lock;
            let tile_reader = tile_lock.read().unwrap();
            let is_assigned = tile_reader.get_state().is_assigned();
            if is_assigned {continue};

            let tile = tile_reader.to_owned();
            let ord_tile = MapNodeEntropyOrdering::from(raw_tile_lock.to_owned());

            queue_writer.push(ord_tile);
            break; // we only want the first uninitialized element
        }
        self.queue_state = QueueState::Initialized;
        wrapped_queue
    }

    pub fn new_with_queue(rules: MapColoringAssigner<MN::Assignment>, map: M) -> Self {
        let mut inst = Self::new(rules, map);
        inst.build_queue();
        inst
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS> + Eq + Hash, M: TileMap<'a, DIMS, MN>> MapColoringJob<'a, DIMS, MN, ThreadsafeNodeRef<'a, DIMS, MN>, M>
where
{
    pub fn assign_map(&mut self) -> &Arc<RwLock<M>> {
        let mut queue_writer = self.queue.write().unwrap();
        let map: &Arc<RwLock<M>> = &self.map;
        let mut map_operator = map.write().unwrap();
        let unassigned = map_operator.get_unassigned();

        let mut enqueued = HashSet::with_capacity(unassigned.len());
        match queue_writer.peek() {
            Some(enqueued_node) => enqueued.insert(enqueued_node.node.position()),
            None => false
        };

        while !queue_writer.is_empty() {
            let assignee = &queue_writer.pop().unwrap().node;
            let raw_node = match assignee {
                MapNodeWrapper::Raw(node) => Arc::new(RwLock::new(node.to_owned())),
                MapNodeWrapper::Arc(node) => node.to_owned(),
                MapNodeWrapper::_Fake(_) => panic!("This should never be reachable!")
            };
            let mut node = raw_node.write().unwrap();
            let mut node_state = &node.get_state().to_owned();
            let node_pos = node.get_position();
            enqueued.remove(&node_pos);
            map_operator.get_unassigned().remove(&node_pos);

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

            let mut node_state = node.get_state();
            node_state = MapNodeState::from(new_assignment);

            let neighbors = map_operator.adjacent(&node_pos);
            let niter = neighbors.into_iter();
            drop(node);

            for raw_neighbor in niter {
                let cast_neighbor: ThreadsafeNodeRef<DIMS, MN> = raw_neighbor;
                // println!("Acquiring lock for neighbor {:?}...", neighbor);
                let mut neighbor_writer = cast_neighbor.write().unwrap();
                let mut neighbor_state = neighbor_writer.get_state();
                let neighbor_pos = neighbor_writer.get_position();

                let neighbor_rule_probas = match &neighbor_writer.get_state() {
                    MapNodeState::Undecided(probas) => probas,
                    MapNodeState::Finalized(_) => continue
                };

                let new_possibilities = self_rule_probas.joint_probability(&neighbor_rule_probas);
                neighbor_state = MapNodeState::from(new_possibilities);
                //println!("Assigned new probas for neighbor {:?}!", neighbor);

                let neigh_pos = neighbor_pos;
                //drop(neighbor_writer);

                let neigh_queued = enqueued.get(&neigh_pos).is_some();

                if !neigh_queued {
                    let wrapped_neighbor = MapNodeEntropyOrdering::from(cast_neighbor.to_owned());
                    queue_writer.push(wrapped_neighbor);
                }
            }
        }

        map
    }

    pub fn queue_and_assign(&mut self) -> &Arc<RwLock<M>> {
        self.build_queue();
        self.assign_map()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::map2d::Map2D;
    use crate::map2Dnode::Map2DNode;
    use crate::position2d::Position2D;
    use super::*;

    #[test]
    fn small_assignment() {
        const TEST_MAP_SIZE: i64 = 10;
        let tile_positions = (0..TEST_MAP_SIZE).cartesian_product(0..TEST_MAP_SIZE);
        let test_tiles = tile_positions.map(
            |(x, y)| Map2DNode::with_possibilities(
                Position2D::new(i64::from(x), i64::from(y)),
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
        let pre_run_state = &job.map.read().unwrap().undecided_tiles.to_owned();
        assert!(pre_run_state.len() > 0);
        job.queue_and_assign();
        let post_run_state = &job.map.read().unwrap().undecided_tiles.to_owned();
        assert_eq!(post_run_state.len(), 0);
    }
}
