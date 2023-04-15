use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use serde::{Serialize, Deserialize};
use crate::map::{MapNode, MapPosition, PositionKey};
use crate::map_node_state::MapNodeState;
use crate::sampler::{DistributionKey, MultinomialDistribution};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map2DNode<'a, K: DistributionKey, P: PositionKey, MP: MapPosition<'a, 2, 8>> {
    pub(crate) position: MP,
    pub(crate) state: MapNodeState<K>,
    pos_phantom: PhantomData<&'a P>
}


impl<'a, K: DistributionKey, P: PositionKey, MP: MapPosition<'a, 2, 8>> PartialOrd<Self> for Map2DNode<'a, K, P, MP> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position.partial_cmp(&other.position)
    }
}

impl<'a, K: DistributionKey, P: PositionKey, MP: MapPosition<'a, 2, 8>> PartialEq<Self> for Map2DNode<'a, K, P, MP> {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
    }
}

impl<'a, K: DistributionKey, P: PositionKey, MP: MapPosition<'a, 2, 8>> Eq for Map2DNode<'a, K, P, MP> {}

impl<'a, K: DistributionKey, P: PositionKey, MP: MapPosition<'a, 2, 8>> Hash for Map2DNode<'a, K, P, MP> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        match &self.state {
            MapNodeState::Undecided(_) => (),
            MapNodeState::Finalized(assignment) => assignment.hash(state),
        }
    }
}

impl<'a, K: DistributionKey + 'a, P: PositionKey, MP: MapPosition<'a, 2, 8> + 'a> MapNode<'a, 2> for Map2DNode<'a, K, P, MP> {
    type PositionKey = P;
    type ReadAs = &'a Self;
    type Position = MP;
    type Assignment = K;

    fn with_possibilities(position: MP, possibilities: MultinomialDistribution<K>) -> Self {
        Self {
            position,
            state: MapNodeState::undecided(possibilities),
            pos_phantom: PhantomData
        }
    }

    fn read_node(&'a self) -> Self::ReadAs { self }

    fn get_position(&self) -> Self::Position {
        let pos = &self.position;
        pos.to_owned()
    }

    fn get_state(&self) -> MapNodeState<Self::Assignment> { self.state.to_owned() }

    fn get_entropy(&self) -> f32 { self.read_node().entropy() }

    fn adjacent<I: IntoIterator<Item=Self::Position>>(&self) -> I {
        let node = &self.read_node();
        let adj = node.adjacent();
        adj
    }
}


impl<'a, K: DistributionKey, P: PositionKey, MP: MapPosition<'a, 2, 8>> Map2DNode<'a, K, P, MP> {

    pub fn with_assignment(position: MP, assignment: K) -> Self {
        Self {
            position,
            state: MapNodeState::finalized(assignment),
            pos_phantom: PhantomData
        }
    }

    pub fn entropy(&self) -> f32 {
        match &self.state {
            MapNodeState::Finalized(_) => f32::INFINITY,
            MapNodeState::Undecided(possibilities) => possibilities.entropy()
        }
    }
}
