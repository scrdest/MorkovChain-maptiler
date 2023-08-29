use std::borrow::Borrow;
use std::ops::{Add};
use serde;
use serde::Serialize;
use crate::adjacency::AdjacencyGenerator;
use crate::position::{PositionKey};
use crate::position::{ConvertibleMapPosition, MapPosition};

#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Position2D<P: PositionKey + Serialize> {
    pub x: P,
    pub y: P
}

impl<P: PositionKey + Serialize> Position2D<P> {
    pub fn new(x: P, y: P) -> Self {
        Self {x, y}
    }
}

impl<P: PositionKey + Serialize> From<(P, P)> for Position2D<P> {
    fn from(value: (P, P)) -> Self {
        Self {
            x: value.0,
            y: value.1
        }
    }
}

impl<P: PositionKey + Serialize> From<Position2D<P>> for (P, P) {
    fn from(value: Position2D<P>) -> (P, P) {
        (value.x, value.y)
    }
}

impl<PA: PositionKey + Add<Output = PA> + Serialize> Add for Position2D<PA> {
    type Output = Position2D<PA>;

    fn add(self, rhs: Self) -> Self::Output {
        Position2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl<'a, P: PositionKey + Serialize + 'a> MapPosition<2> for Position2D<P> {
    type Key = P;

    fn get_dims(&self) -> [Self::Key; 2] {
        [self.x, self.y]
    }

    fn from_dims(dims: [Self::Key; 2]) -> Self {
        let dim_x = dims[0];
        let dim_y = dims[1];

        Position2D::new(dim_x, dim_y)
    }

    fn adjacents<BS: Borrow<Self>, AG: AdjacencyGenerator<2, Input=BS>>(borrowed: BS) -> AG::Output {
        let cast_self: BS = borrowed;
        AG::adjacents(cast_self)
    }
}

impl<P: PositionKey + Into<u32> + Serialize> ConvertibleMapPosition<2, u32, Position2D<u32>> for Position2D<P> {
    fn convert(self) -> Position2D<u32> {
        let dimarray: [Self::Key; 2] = <Position2D<P> as MapPosition<2>>::get_dims(&self);
        let new_arr = dimarray.map(|d| {
            let new_dim: u32 = d.into();
            new_dim
        });

        Position2D::new(
            new_arr[0],
            new_arr[1]
        )
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CompactMapPosition<P: PositionKey + Serialize> {
    pos: [P; 2]
}

impl<P: PositionKey + Serialize> From<Position2D<P>> for CompactMapPosition<P> {
    fn from(value: Position2D<P>) -> Self {
        CompactMapPosition {
            pos: [value.x, value.y]
        }
    }
}

impl<P: PositionKey + Serialize> MapPosition<2> for CompactMapPosition<P> {
    type Key = P;

    fn get_dims(&self) -> [Self::Key; 2] {
        self.pos
    }

    fn from_dims(dims: [Self::Key; 2]) -> Self {
        Self {
            pos: dims
        }
    }

    fn adjacents<BS: Borrow<Self>, AG: AdjacencyGenerator<2, Input=BS>>(borrowed: BS) -> AG::Output {
        let cast_self: BS = borrowed;
        AG::adjacents(cast_self)
    }
}

impl<P: PositionKey + Serialize> ConvertibleMapPosition<2, P, CompactMapPosition<P>> for Position2D<P> {
    fn convert(self) -> CompactMapPosition<P> {
        let dimarray: [Self::Key; 2] = <Position2D<P> as MapPosition<2>>::get_dims(&self);
        CompactMapPosition {
            pos: dimarray
        }
    }
}
