use std::hash::Hash;
use std::ops::Add;
use arrayvec::ArrayVec;

pub trait PositionKey: Copy + Clone + Add<Output = Self> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Zero + num::One + num::Bounded {}
// blanket impl for any good types
impl<P: Copy + Clone + Add<Output = P> + PartialOrd + Ord + Eq + Hash + num::Num + num::ToPrimitive + num::Zero + num::One + num::Bounded> PositionKey for P {}


pub trait MapPosition<const DIMS: usize, const ADJACENTS: usize>: Eq + Hash + Sized + Copy + Clone {
    type Key: PositionKey;

    fn get_dims(&self) -> [Self::Key; DIMS];
    fn from_dims(dims: [Self::Key; DIMS]) -> Self;

    fn adjacents(&self) -> ArrayVec<Self, ADJACENTS>;
}

pub trait ConvertibleMapPosition<const DIMS: usize, const ADJACENTS: usize, T>: MapPosition<DIMS, ADJACENTS> {
    type ConvertsTo: MapPosition<DIMS, ADJACENTS, Key=T>;

    fn convert(self) -> Self::ConvertsTo;
}