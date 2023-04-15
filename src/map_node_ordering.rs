use std::cmp::Ordering;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use crate::map2d::MapNodeWrapper;
use crate::map::{MapNode};

#[derive(Serialize, Deserialize)]
pub struct MapNodeEntropyOrdering<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> {
    pub node: MapNodeWrapper<'a, DIMS, MN>
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> From<MN> for MapNodeEntropyOrdering<'a, DIMS, MN> {
    fn from(value: MN) -> Self {
        Self {
            node: MapNodeWrapper::Raw(value.clone())
        }
    }
}

impl<'a, MN: MapNode<'a, 2>> From<Arc<RwLock<MN>>> for MapNodeEntropyOrdering<'a, 2, MN> {
    fn from(value: Arc<RwLock<MN>>) -> Self {
        Self {
            node: MapNodeWrapper::Arc(value.clone())
        }
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> PartialEq<Self> for MapNodeEntropyOrdering<'a, DIMS, MN> {
    fn eq(&self, other: &Self) -> bool {
        let my_entropy = match &self.node {
            MapNodeWrapper::Raw(node_data) => node_data.get_entropy(),
            MapNodeWrapper::Arc(node_data) => node_data.read().unwrap().get_entropy(),
            MapNodeWrapper::_Fake(_) => panic!("This should never be reachable!")
        };

        let other_entropy = match &other.node {
            MapNodeWrapper::Raw(node_data) => node_data.get_entropy(),
            MapNodeWrapper::Arc(node_data) => node_data.read().unwrap().get_entropy(),
            MapNodeWrapper::_Fake(_) => panic!("This should never be reachable!")
        };

        my_entropy == other_entropy
    }
}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> Eq for MapNodeEntropyOrdering<'a, DIMS, MN> {}

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> PartialOrd for MapNodeEntropyOrdering<'a, DIMS, MN> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let my_entropy = match &self.node {
            MapNodeWrapper::Raw(node_data) => node_data.get_entropy(),
            MapNodeWrapper::Arc(node_data) => node_data.read().unwrap().get_entropy(),
            MapNodeWrapper::_Fake(_) => panic!("This should never be reachable!")
        };

        let other_entropy = match &other.node {
            MapNodeWrapper::Raw(node_data) => node_data.get_entropy(),
            MapNodeWrapper::Arc(node_data) => node_data.read().unwrap().get_entropy(),
            MapNodeWrapper::_Fake(_) => panic!("This should never be reachable!")
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

impl<'a, const DIMS: usize, MN: MapNode<'a, DIMS>> Ord for MapNodeEntropyOrdering<'a, DIMS, MN> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
