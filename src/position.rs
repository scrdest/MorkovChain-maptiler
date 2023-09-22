use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add};
use serde::Serialize;
use crate::adjacency::AdjacencyGenerator;

pub trait PositionKey: Debug + Copy + Clone + Add<Output = Self> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Zero + num::One + num::Bounded + num::CheckedAdd + num::CheckedSub {}
// blanket impl for any good types
impl<P: Debug + Copy + Clone + Add<Output = P> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Zero + num::One + num::Bounded + num::CheckedAdd + num::CheckedSub> PositionKey for P {}


pub trait MapPosition<const DIMS: usize>: Eq + Hash + Sized + Copy + Clone + Borrow<Self> + Serialize + Debug {
    type Key: PositionKey;

    fn get_dims(&self) -> [Self::Key; DIMS];
    fn from_dims(dims: [Self::Key; DIMS]) -> Self;
    fn adjacents<BS: Borrow<Self>, AG: AdjacencyGenerator<2, Input=BS>>(borrowed: BS) -> AG::Output;
}

pub trait ConvertibleMapPosition<const DIMS: usize, T, OUT: MapPosition<DIMS, Key=T>>: MapPosition<DIMS> {
    fn convert(self) -> OUT;
}

pub trait IsType<P> {}
impl<P> IsType<P> for P {}

