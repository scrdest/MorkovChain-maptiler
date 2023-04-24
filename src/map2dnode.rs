use std::sync::{Arc, RwLock};
use std::cmp::Ordering;
use std::marker::PhantomData;
use serde;
use crate::adjacency::AdjacencyGenerator;
use crate::position::{MapPosition};
use crate::sampler::{DistributionKey, MultinomialDistribution};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum MapNodeState<K: DistributionKey> {
    Undecided(MultinomialDistribution<K>),
    Finalized(K)
}

impl<K: DistributionKey> MapNodeState<K> {
    pub fn undecided(possibilities: MultinomialDistribution<K>) -> Self {
        Self::Undecided(possibilities)
    }

    pub fn finalized(assignment: K) -> Self {
        Self::Finalized(assignment)
    }

    pub fn is_assigned(&self) -> bool {
        match self {
            Self::Undecided(_) => false,
            Self::Finalized(_) => true
        }
    }
}

impl<K: DistributionKey> From<MultinomialDistribution<K>> for MapNodeState<K> {
    fn from(value: MultinomialDistribution<K>) -> Self {
        Self::undecided(value)
    }
}

impl<K: DistributionKey> From<K> for MapNodeState<K> {
    fn from(value: K) -> Self {
        Self::finalized(value)
    }
}


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Map2DNode<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> {
    pub(crate) position: MP,
    pub(crate) state: MapNodeState<K>,
    adjacency_phantom: PhantomData<AG>,
}

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> Map2DNode<AG, K, MP>
{
    pub fn with_possibilities(position: MP, possibilities: MultinomialDistribution<K>) -> Self {
        Self {
            position,
            state: MapNodeState::undecided(possibilities),
            adjacency_phantom: PhantomData
        }
    }

    pub fn with_assignment(position: MP, assignment: K) -> Self {
        Self {
            position,
            state: MapNodeState::finalized(assignment),
            adjacency_phantom: PhantomData
        }
    }

    pub fn entropy(&self) -> f32 {
        match &self.state {
            MapNodeState::Finalized(_) => f32::INFINITY,
            MapNodeState::Undecided(possibilities) => possibilities.entropy()
        }
    }

    pub fn get_position(&self) -> MP {
        self.position
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum MapNodeWrapper<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> {
    Raw(Map2DNode<AG, K, MP>),
    Arc(Arc<RwLock<Map2DNode<AG, K, MP>>>)
}

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> MapNodeWrapper<AG, K, MP>
{
    pub fn position(&self) -> MP {
        match self {
            Self::Raw(node) => node.position,
            Self::Arc(arc_node) => arc_node.read().unwrap().position
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MapNodeEntropyOrdering<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> {
    pub node: MapNodeWrapper<AG, K, MP>
}

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> From<Map2DNode<AG, K, MP>> for MapNodeEntropyOrdering<AG, K, MP> {
    fn from(value: Map2DNode<AG, K, MP>) -> Self {
        Self {
            node: MapNodeWrapper::Raw(value.clone())
        }
    }
}

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> From<Arc<RwLock<Map2DNode<AG, K, MP>>>> for MapNodeEntropyOrdering<AG, K, MP> {
    fn from(value: Arc<RwLock<Map2DNode<AG, K, MP>>>) -> Self {
        Self {
            node: MapNodeWrapper::Arc(value.clone())
        }
    }
}

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> PartialEq<Self> for MapNodeEntropyOrdering<AG, K, MP> {
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

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> Eq for MapNodeEntropyOrdering<AG, K, MP> {}

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> PartialOrd for MapNodeEntropyOrdering<AG, K, MP> {
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

impl<AG: AdjacencyGenerator<2>, K: DistributionKey, MP: MapPosition<2>> Ord for MapNodeEntropyOrdering<AG, K, MP> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub type ThreadsafeNodeRef<AG, K, MP> = Arc<RwLock<Map2DNode<AG, K, MP>>>;
