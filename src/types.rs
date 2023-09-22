use std::collections::HashMap;
use crate::directions::{CardinallyDirected};
use crate::DistributionKey;
use crate::sampler::MultinomialDistribution;
use serde::{Serialize, Deserialize};

// use ahash::AHashMap;

pub type GridMapDs<K, V> = HashMap<K, V>;
// pub type GridMapDs<K, V> = AHashMap<K, V>;


pub type DirectedMultinomialDistribution<DK> = CardinallyDirected<MultinomialDistribution<DK>>;

#[derive(Clone, Serialize, Deserialize)]
pub enum PossiblyDirectedMultinomialDistribution<DK: DistributionKey>
{
    Directed(DirectedMultinomialDistribution<DK>),
    Undirected(MultinomialDistribution<DK>)
}

impl<DK: DistributionKey> From<MultinomialDistribution<DK>> for PossiblyDirectedMultinomialDistribution<DK> {
    fn from(value: MultinomialDistribution<DK>) -> Self {
        Self::Undirected(value)
    }
}

impl<DK: DistributionKey> From<DirectedMultinomialDistribution<DK>> for PossiblyDirectedMultinomialDistribution<DK> {
    fn from(value: DirectedMultinomialDistribution<DK>) -> Self {
        Self::Directed(value)
    }
}
