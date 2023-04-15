use std::collections::HashMap;
use std::ops::{Add};
use std::hash::Hash;
use arrayvec::ArrayVec;
use serde::{Serialize};
use crate::map2d::ThreadsafeNodeRef;
use crate::map_node_state::MapNodeState;
use crate::sampler::{DistributionKey, MultinomialDistribution};

pub trait PositionKey: Copy + Clone + Add<Output = Self> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Bounded {}
impl<P: Copy + Clone + Add<Output = P> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Bounded> PositionKey for P {}

pub(crate) trait FromArrayVec<const DIMS: usize> {
    type Item;

    fn from_array_vec(arrayvec: ArrayVec<Self::Item, DIMS>) -> Self;
}

pub(crate) trait IsArrayVec<P: Sized, const DIMS: usize>: IntoIterator<Item=P> + FromArrayVec<DIMS, Item=P> {
    fn concrete(self) -> ArrayVec<P, DIMS>;
    fn from_arr(arr: [P; DIMS]) -> Self;
}

impl<const DIMS: usize, P> FromArrayVec<DIMS> for ArrayVec<P, DIMS> {
    type Item = P;
    fn from_array_vec(arrayvec: ArrayVec<Self::Item, DIMS>) -> Self { arrayvec /* categorical identity. */ }
}

impl<P, const DIMS: usize> IsArrayVec<P, DIMS> for ArrayVec<P, DIMS> {
    fn concrete(self) -> Self { self }
    fn from_arr(arr: [P; DIMS]) -> Self { Self::from(arr) }
}

pub trait MapPosition<'a, const DIMS: usize, const ADJACENTS: usize>: Eq + Hash + PartialOrd + Clone + Serialize + FromArrayVec<DIMS> {
    type PositionKey: PositionKey;
    type DimArray: IsArrayVec<Self::PositionKey, DIMS>;
    type AdjacentsArray: IsArrayVec<Self, ADJACENTS>;

    fn to_dim_array(self) -> Self::DimArray;
    fn adjacent(&self) -> Self::AdjacentsArray;
    fn from_dim_array(dimarr: &Self::DimArray) -> Self;
    fn get_dim<IX: Into<u8>>(&self, idx: IX) -> Option<Self::PositionKey>;
}

pub trait BoundedMapPosition<'a, const DIMS: usize, const ADJACENTS: usize> : MapPosition<'a, DIMS, ADJACENTS> {
    fn min_position() -> Self;
    fn max_position() -> Self;
    fn any_out_of_bounds<I: IntoIterator<Item=<Self as MapPosition<'a, DIMS, ADJACENTS>>::PositionKey>>(dims: I) -> bool;
}

pub trait SquishyMapPosition<'a, const DIMS: usize, const ADJACENTS: usize, P: PositionKey, S: num::Bounded + PositionKey> : BoundedMapPosition<'a, DIMS, ADJACENTS> {
    /* Position keys can be interpolated into the (also bounded) domain of a different position key type */
    type Output: BoundedMapPosition<'a, DIMS, ADJACENTS, PositionKey=S>;

    fn squish(&self) -> Self::Output;
    fn output_span() -> S;
}

pub trait MapNode<'a, const DIMS: usize>: PartialOrd + Clone + Serialize + Eq + Hash {
    type PositionKey: PositionKey;
    type ReadAs;
    type Position: MapPosition<'a, DIMS, 8> + FromArrayVec<DIMS>;
    type Assignment: DistributionKey;

    fn with_possibilities(position: Self::Position, possibilities: MultinomialDistribution<Self::Assignment>) -> Self;
    fn read_node(&'a self) -> Self::ReadAs;
    fn get_position(&'a self) -> Self::Position;
    fn get_state(&self) -> MapNodeState<Self::Assignment>;
    fn get_entropy(&self) -> f32;
    fn adjacent<I: IntoIterator<Item=Self::Position>>(&self) -> I;
}

pub trait TileMap<'a, const DIMS: usize, MN: MapNode<'a, DIMS> + 'a> {
    type TileContainer: IntoIterator<Item=ThreadsafeNodeRef<'a, DIMS, MN>>;
    type AdjacentsArray: IntoIterator<Item=ThreadsafeNodeRef<'a, DIMS, MN>>;
    type PositionIndex;
    type UndecidedIndex;
    type NodeReadAs: AsRef<MN>;

    fn parse_tiles<I: IntoIterator<Item=MN>>(tiles: &I) -> (Self::PositionIndex, Self::UndecidedIndex, MN::Position, MN::Position);

    fn build<I: IntoIterator<Item=MN>>(
        tiles: &I,
        position_index: Self::PositionIndex,
        undecided_tiles: Self::UndecidedIndex,
        min_pos: MN::Position,
        max_pos: MN::Position
    ) -> Self;

    fn adjacent(&self, position: &MN::Position) -> Self::AdjacentsArray;

    fn get_min_pos(&self) -> &MN::Position;
    fn get_max_pos(&self) -> &MN::Position;

    fn get_node_by_pos(&self, key: &MN::Position) -> Option<Self::NodeReadAs>;

    fn get_tiles(&self) -> Self::TileContainer;
    fn get_unassigned(self) -> HashMap<MN::Position, ThreadsafeNodeRef<'a, DIMS, MN>>;

    fn read_access(&self, node: &MN) -> Self::NodeReadAs;
}

