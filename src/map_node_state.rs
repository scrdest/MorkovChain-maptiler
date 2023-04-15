use serde::{Serialize, Deserialize};
use crate::sampler::{DistributionKey, MultinomialDistribution};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
