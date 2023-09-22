use std::marker::PhantomData;
use arrayvec::ArrayVec;
use num::{CheckedAdd, CheckedSub, One, Zero};
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

                let mut pos_buffer = bound_position.get_dims();
                let new_dim = pos_buffer[dim]
                    .checked_add(&offset)
                    .and_then(|nu_dim| nu_dim.checked_sub(&type_unity));

                if new_dim.is_none() { continue };

                pos_buffer[dim] = new_dim.unwrap();

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

        for x_dim in x_range {
            let new_x = pos_array[0]
            .checked_add(&x_dim) // should be saturating I guess?
            .and_then(|nu_x|
                nu_x.checked_sub(&type_unity)
            );

            if new_x.is_none() { continue };

            let y_range = num::range(
                MP::Key::zero(),
                type_three
            );

            for y_dim in y_range {
                if x_dim.is_one() && y_dim.is_one() {
                    continue // would just return input pos
                };

                let new_y = pos_array[1]
                .checked_add(&y_dim) // should be saturating I guess?
                .and_then(|nu_y|
                    nu_y.checked_sub(&type_unity)
                );

                if new_y.is_none() { continue };

                let new_pos = MP::from_dims([
                    new_x.unwrap(),
                    new_y.unwrap()
                ]);

                adjacents.push(new_pos);
            }
        }

        adjacents
    }
}