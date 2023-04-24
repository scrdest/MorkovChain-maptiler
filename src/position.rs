use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::{Add};
use crate::adjacency::AdjacencyGenerator;

pub trait PositionKey: Copy + Clone + Add<Output = Self> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Zero + num::One + num::Bounded {}
// blanket impl for any good types
impl<P: Copy + Clone + Add<Output = P> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Zero + num::One + num::Bounded> PositionKey for P {}


pub trait MapPosition<const DIMS: usize>: Eq + Hash + Sized + Copy + Clone + Borrow<Self> {
    type Key: PositionKey;

    fn get_dims(&self) -> [Self::Key; DIMS];
    fn from_dims(dims: [Self::Key; DIMS]) -> Self;
    fn adjacents<BS: Borrow<Self>, AG: AdjacencyGenerator<2, Input=BS>>(borrowed: BS) -> AG::Output;
}

pub trait ConvertibleMapPosition<const DIMS: usize, T>: MapPosition<DIMS> {
    type ConvertsTo: MapPosition<DIMS, Key=T>;

    fn convert(self) -> Self::ConvertsTo;
}
