use std::marker::PhantomData;
use arrayvec::ArrayVec;
use num::{One, Zero};
use crate::position::MapPosition;


pub trait AdjacencyGenerator<const DIMS: usize>: Sized + Copy + Clone {
    type Input: MapPosition<DIMS>;
    type Output: IntoIterator<Item=Self::Input>;

    fn adjacents(bound_position: Self::Input) -> Self::Output;
}


#[derive(Copy, Clone)]
pub struct CardinalAdjacencyGenerator<MP: MapPosition<2>> {
    bound_position: PhantomData<MP>
}

impl<MP: MapPosition<2>> AdjacencyGenerator<2> for CardinalAdjacencyGenerator<MP>
{
    type Input = MP;
    type Output = ArrayVec<MP, 4>;

    fn adjacents(bound_position: Self::Input) -> Self::Output {
        let mut adjacents: ArrayVec::<MP, 4> = ArrayVec::new();
        let type_unity: MP::Key = num::one();
        let type_three: MP::Key = type_unity + type_unity + type_unity;

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

                let mut pos_buffer = bound_position.get_dims();
                pos_buffer[dim] = pos_buffer[dim] + true_offset;

                let new_pos = MP::from_dims(pos_buffer);

                adjacents.push(new_pos);
            }
        }
        adjacents
    }
}

#[derive(Copy, Clone)]
pub struct OctileAdjacencyGenerator<MP: MapPosition<2>> {
    bound_position: PhantomData<MP>
}

impl<MP: MapPosition<2>> AdjacencyGenerator<2> for OctileAdjacencyGenerator<MP>
{
    type Input = MP;
    type Output = ArrayVec<MP, 8>;

    fn adjacents(bound_position: Self::Input) -> Self::Output {
        let mut adjacents: ArrayVec<MP, 8> = ArrayVec::new();

        let type_unity: MP::Key = MP::Key::one();
        let type_three = type_unity + type_unity + type_unity;
        let pos_array = bound_position.get_dims();

        let x_range = num::range(
            MP::Key::zero(),
            type_three
        );

        for raw_x_dim in x_range {
            let x_dim = raw_x_dim - type_unity;

            let y_range = num::range(
                MP::Key::zero(),
                type_three
            );

            for raw_y_dim in y_range {
                if raw_x_dim.is_one() && raw_y_dim.is_one() {
                    continue
                };
                let y_dim = raw_y_dim - type_unity;
                let new_pos = MP::from_dims([
                    pos_array[0] + x_dim,
                    pos_array[1] + y_dim
                ]);
                adjacents.push(new_pos);
            }
        }

        adjacents
    }
}