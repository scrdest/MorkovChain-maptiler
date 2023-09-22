use std::fmt::Debug;
use std::hash::Hash;
use serde::{Serialize, Deserialize};
use crate::DistributionKey;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Directions2d {
    NORTH,
    NORTHEAST,
    EAST,
    SOUTHEAST,
    SOUTH,
    SOUTHWEST,
    WEST,
    NORTHWEST
}


/// A complete parcel of directed values for 2d cardinal directions
pub trait Cardinal2dDirectionVariant<T> {
    fn north(&self) -> &T;
    fn east(&self) -> &T;
    fn south(&self) -> &T;
    fn west(&self) -> &T;

    fn value_for_cardinal(&self, card: &Directions2d) -> Option<&T> {
        match card {
            Directions2d::NORTH => Some(self.north()),
            Directions2d::EAST => Some(self.east()),
            Directions2d::SOUTH => Some(self.south()),
            Directions2d::WEST => Some(self.west()),
            _ => None
        }
    }
}

/// A complete parcel of directed values for 2d adjacents (i.e. Chebyshev distance 1 on a square grid)
pub trait Octile2dDirectionVariant<T>: Cardinal2dDirectionVariant<T> {
    fn northeast(&self) -> &T;
    fn southeast(&self) -> &T;
    fn southwest(&self) -> &T;
    fn northwest(&self) -> &T;

    fn value_for_octile(&self, card: &Directions2d) -> Option<&T> {
        Self::value_for_cardinal(self, card).or(
            match card {
                Directions2d::NORTHEAST => Some(self.northeast()),
                Directions2d::SOUTHEAST => Some(self.southeast()),
                Directions2d::SOUTHWEST => Some(self.southwest()),
                Directions2d::NORTHWEST => Some(self.northwest()),
                _ => None
            }
        )
    }
}


#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CardinallyDirected<T: Clone> {
    north: T,
    east: T,
    south: T,
    west: T
}

impl<T: Clone> Cardinal2dDirectionVariant<T> for CardinallyDirected<T> {
    fn north(&self) -> &T {
        &self.north
    }

    fn east(&self) -> &T {
        &self.east
    }

    fn south(&self) -> &T {
        &self.south
    }

    fn west(&self) -> &T {
        &self.west
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq, Default)]
pub struct CardinallyDirectedDk<DK: DistributionKey> {
    north: DK,
    east: DK,
    south: DK,
    west: DK,
    identity: Option<DK>
}

impl<DK: DistributionKey> Cardinal2dDirectionVariant<DK> for CardinallyDirectedDk<DK> {
    fn north(&self) -> &DK {
        &self.north
    }

    fn east(&self) -> &DK {
        &self.east
    }

    fn south(&self) -> &DK {
        &self.south
    }

    fn west(&self) -> &DK {
        &self.west
    }
}
