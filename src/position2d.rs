use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use arrayvec::ArrayVec;
use std::ops::Add;
use num::{Bounded, Zero};
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use crate::map::{BoundedMapPosition, FromArrayVec, MapPosition, PositionKey, SquishyMapPosition};

impl<'a, P: PositionKey> Position2D<'a, P> {
    pub fn new(x: P, y: P) -> Self {
        Self {x, y, life_phantom: PhantomData}
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Position2D<'a, P: PositionKey + 'a> {
    pub x: P,
    pub y: P,
    life_phantom: PhantomData<&'a Self>
}

impl<'a, P: PositionKey> From<(P, P)> for Position2D<'a, P> {
    fn from(value: (P, P)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            life_phantom: PhantomData
        }
    }
}

impl<'a, P: PositionKey> Into<(P, P)> for Position2D<'a, P> {
    fn into(self) -> (P, P) {
        (self.x, self.y)
    }
}

impl<'a, P: PositionKey + Debug> FromArrayVec<2> for Position2D<'a, P> {
    type Item = P;

    fn from_array_vec(arrayvec: ArrayVec<Self::Item, 2>) -> Self {
        let array_data = arrayvec.into_inner().unwrap();
        Self::from((array_data[0], array_data[1]))
    }
}


impl<'a, P: PositionKey> Position2D<'a, P> {
    pub fn adjacents_cardinal(&self) -> ArrayVec<Position2D<'a, P>, 4> {
        let mut adjacents: ArrayVec::<Position2D<'a, P>, 4> = ArrayVec::new();
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

                let new_pos = Position2D {
                    x: pos_buffer[0].into(),
                    y: pos_buffer[1].into(),
                    life_phantom: PhantomData
                };

                adjacents.push(new_pos);
            }
        }

        adjacents
    }

    pub fn adjacents_octile(&self) -> ArrayVec<Self, 8> {
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
                    y: self.y + y_dim,
                    life_phantom: PhantomData
                };
                adjacents.push(new_pos);
            }
        }

        adjacents
    }
}

impl<'a, P: PositionKey> Bounded for Position2D<'a, P> {
    fn min_value() -> Self {
        let min = P::min_value();
        Self::from((min, min))
    }

    fn max_value() -> Self {
        let max = P::max_value();
        Self::from((max, max))
    }
}

impl<'a, P: PositionKey> Zero for Position2D<'a, P> {
    fn zero() -> Self {
        let zero = P::zero();
        Self::from((zero, zero))
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }
}

impl<'a, P: PositionKey> Add<Self> for Position2D<'a, P> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from((
            self.x + rhs.x,
            self.y + rhs.y
        ))
    }
}

impl<'de: 'a, 'a, P: PositionKey + Serialize + for<'d> Deserialize<'d> + Debug> MapPosition<'a, 2, 8> for Position2D<'a, P> {
    type PositionKey = P;
    type DimArray = ArrayVec<P, 2>;
    type AdjacentsArray = ArrayVec<Self, 8>;

    fn to_dim_array(self) -> ArrayVec<P, 2> {
        ArrayVec::from([self.x, self.y])
    }

    fn adjacent(&self) -> Self::AdjacentsArray {
        self.adjacents_octile()
    }

    fn from_dim_array(dim_arr: &Self::DimArray) -> Self {
        Self {
            x: dim_arr[0],
            y: dim_arr[1],
            life_phantom: PhantomData
        }
    }

    fn get_dim<IX: Into<u8>>(&self, idx: IX) -> Option<P> {
        match idx.into() {
            0u8 => Some(self.x),
            1u8 => Some(self.y),
            _ => None
        }
    }
}

impl<'a, P: PositionKey + Debug + num::Bounded + Serialize + for<'d> Deserialize<'d>> BoundedMapPosition<'a, 2, 8> for Position2D<'a, P> {
    fn min_position() -> Self {
        Self {
            x: P::min_value(),
            y: P::min_value(),
            life_phantom: PhantomData
        }
    }

    fn max_position() -> Self {
        Self {
            x: P::max_value(),
            y: P::max_value(),
            life_phantom: PhantomData
        }
    }

    fn any_out_of_bounds<I: IntoIterator<Item=P>>(dims: I) -> bool {
        let min_val = P::min_value();
        let max_val = P::max_value();
        for dimval in dims {
            if dimval < min_val { return true; }
            if dimval > max_val { return true; }
        }
        return false
    }
}

impl<
    'de: 'a,
    'a,
    P: PositionKey + Debug + num::Bounded + Serialize + DeserializeOwned,
    S: PositionKey + Debug + num::Bounded + Serialize + DeserializeOwned + num::Num + From<P> + 'a
> SquishyMapPosition<'a, 2, 8, P, S> for Position2D<'a, P> {

    type Output = Position2D<'a, S>;

    fn squish(&self) -> Self::Output {
        let min_src = P::min_value();
        let max_src = P::max_value();
        let span_src = max_src - min_src;

        let span_trg = Self::output_span();

        let curr_x = self.x;
        let curr_y = self.y;

        // 0 @ [-50, 50] to [0, 100] => 50
        // 0 @ [0, 100] to [50, 100] => 50
        // 0 @ [0, 100] to [-50, 50] => -50
        // 50 @ [0, 100] to [-50, 50] => 0
        let new_x: S = S::from((curr_x - min_src) / span_src) * span_trg;
        let new_y: S = S::from((curr_y - min_src) / span_src) * span_trg;

        Self::Output::from((new_x, new_y))
    }

    fn output_span() -> S {
        S::max_value() - S::min_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_vector_addition_works_positives() {
        let pos_a = Position2D { x: 5, y: 3, life_phantom: PhantomData };
        let pos_b = Position2D { x: 2, y: 6, life_phantom: PhantomData };
        let result_pos = pos_a + pos_b;
        assert_eq!(result_pos.x, 7);
        assert_eq!(result_pos.y, 9);
    }

    #[test]
    fn position_vector_addition_works_one_negative() {
        let pos_a = Position2D { x: -5, y: -3, life_phantom: PhantomData };
        let pos_b = Position2D { x: 2, y: 6, life_phantom: PhantomData };
        let result_pos = pos_a + pos_b;
        assert_eq!(result_pos.x, -3);
        assert_eq!(result_pos.y, 3);
    }

    #[test]
    fn adjacents_cardinal_sane() {
        let pos = Position2D { x: 2, y: 6, life_phantom: PhantomData };
        let results = pos.adjacents_cardinal();
        assert_eq!(results[0], Position2D { x: 1, y: 6, life_phantom: PhantomData });
        assert_eq!(results[1], Position2D { x: 3, y: 6, life_phantom: PhantomData });
        assert_eq!(results[2], Position2D { x: 2, y: 5, life_phantom: PhantomData });
        assert_eq!(results[2], Position2D { x: 2, y: 7, life_phantom: PhantomData });
    }

    #[test]
    fn serde_pos() {
        let pos = Position2D { x: 2, y: 6, life_phantom: PhantomData };
        let results = serde_json::to_string(&pos).unwrap();
        assert!(results.len() > 0)
    }
}
