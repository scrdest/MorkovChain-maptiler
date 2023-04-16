use std::ops::Add;
use arrayvec::ArrayVec;
use serde;
use crate::position::PositionKey;
use crate::position::{ConvertibleMapPosition, MapPosition};

#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Position2D<P: PositionKey> {
    pub x: P,
    pub y: P
}

impl<P: PositionKey> Position2D<P> {
    pub fn new(x: P, y: P) -> Self {
        Self {x, y}
    }
}

impl<P: PositionKey> From<(P, P)> for Position2D<P> {
    fn from(value: (P, P)) -> Self {
        Self {
            x: value.0,
            y: value.1
        }
    }
}

impl<P: PositionKey> Into<(P, P)> for Position2D<P> {
    fn into(self) -> (P, P) {
        (self.x, self.y)
    }
}

impl<PA: PositionKey + Add<Output = PA>> Add for Position2D<PA> {
    type Output = Position2D<PA>;

    fn add(self, rhs: Self) -> Self::Output {
        Position2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl<P: PositionKey> MapPosition<2, 4> for Position2D<P> {
    type Key = P;

    fn get_dims(&self) -> [Self::Key; 2] {
        [self.x, self.y]
    }

    fn from_dims(dims: [Self::Key; 2]) -> Self {
        let dim_x = dims[0];
        let dim_y = dims[1];

        Position2D::new(dim_x, dim_y)
    }

    fn adjacents(&self) -> ArrayVec<Self, 4> {
        let mut adjacents: ArrayVec::<Self, 4> = ArrayVec::new();
        let type_unity: P = num::one();
        let type_three: P = type_unity + type_unity + type_unity;

        for dim in 0..2 {

            let offset_range = num::range(
                 num::zero(),
                 type_three
            );

            for offset in offset_range {
                if offset == type_unity {
                    continue
                };
                let true_offset = offset - type_unity;

                let mut pos_buffer = [self.x, self.y];
                pos_buffer[dim] = pos_buffer[dim] + true_offset;

                let new_pos = <Self as MapPosition<2, 4>>::from_dims(pos_buffer);

                adjacents.push(new_pos);
            }
        }

        adjacents
    }
}

impl<P: PositionKey> MapPosition<2, 8> for Position2D<P> {
    type Key = P;

    fn get_dims(&self) -> [Self::Key; 2] {
        [self.x, self.y]
    }

    fn from_dims(dims: [Self::Key; 2]) -> Self {
        let dim_x = dims[0];
        let dim_y = dims[1];

        Position2D::new(dim_x, dim_y)
    }

    fn adjacents(&self) -> ArrayVec<Self, 8> {
        let mut adjacents: ArrayVec<Self, 8> = ArrayVec::new();

        let type_unity: P = P::one();
        let type_three = type_unity + type_unity + type_unity;

        let x_range = num::range(
            P::zero(),
            type_three
        );

        for raw_x_dim in x_range {
            let x_dim = raw_x_dim - type_unity;

            let y_range = num::range(
                P::zero(),
                type_three
            );

            for raw_y_dim in y_range {
                if raw_x_dim.is_one() && raw_y_dim.is_one() {
                    continue
                };
                let y_dim = raw_y_dim - type_unity;
                let new_pos = Position2D {
                    x: self.x + x_dim,
                    y: self.y + y_dim
                };
                adjacents.push(new_pos);
            }
        }

        adjacents
    }
}

impl<P: PositionKey + Into<u32>> ConvertibleMapPosition<2, 4, u32> for Position2D<P> {
    type ConvertsTo = Position2D<u32>;

    fn convert(self) -> Self::ConvertsTo {
        let dimarray: [Self::Key; 2] = <Position2D<P> as MapPosition<2, 4>>::get_dims(&self);
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

impl<P: PositionKey + Into<u32>> ConvertibleMapPosition<2, 8, u32> for Position2D<P> {
    type ConvertsTo = Position2D<u32>;

    fn convert(self) -> Self::ConvertsTo {
        let dimarray: [Self::Key; 2] = <Position2D<P> as MapPosition<2, 8>>::get_dims(&self);
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
