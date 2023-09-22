use serde;
use crate::position::PositionKey;

trait IsBitPacked {
    type Data;

    fn to_raw(self) -> Self::Data;
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Default, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
struct BitPackedData<D>(D);

impl<T> IsBitPacked for BitPackedData<T> {
    type Data = T;

    fn to_raw(self) -> Self::Data {
        self.0
    }
}

trait BitPackablePair<O> {
    type Left;
    type Right;
    type Wrapped: IsBitPacked<Data=O>;

    fn raw_wrapped(self) -> O;

    fn pack_val(left: Self::Left, right: Self::Right) -> Self::Wrapped;

    fn pack_tup(tup: (Self::Left, Self::Right)) -> Self::Wrapped {
        Self::pack_val(tup.0, tup.1)
    }

    fn pack_val_raw(left: Self::Left, right: Self::Right) -> O {
        Self::pack_val(left, right).to_raw()
    }

    fn pack_tup_raw(val: (Self::Left, Self::Right)) -> O {
        Self::pack_tup(val).to_raw()
    }

    fn unpack_raw(val: O) -> (Self::Left, Self::Right);

    fn unpack(val: Self::Wrapped) -> (Self::Left, Self::Right) {
        Self::unpack_raw(val.to_raw())
    }

}


impl BitPackablePair<u64> for (u32, u32) {
    type Left = u32;
    type Right = u32;
    type Wrapped = BitPackedData<u64>;

    fn raw_wrapped(self) -> u64 {
        Self::pack_tup(self).0
    }

    fn pack_val(left: Self::Left, right: Self::Right) -> Self::Wrapped {
        let left_pack = u64::from(left) << 32;
        let right_pack = u64::from(right);
        let total_pack = left_pack | right_pack;
        BitPackedData(total_pack)
    }

    fn unpack_raw(val: u64) -> (Self::Left, Self::Right) {
        let mask = u64::from(u32::MAX);

        let raw_left = (val & (mask << 32)) >> 32;
        let raw_right = val & mask;

        let left = u32::try_from(raw_left).unwrap();
        let right = u32::try_from(raw_right).unwrap();

        (left, right)
    }
}


#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Default, serde::Serialize, serde::Deserialize)]
struct BitPackedPosition<T> {
    pos: BitPackedData<T>
}


impl<T> BitPackedPosition<T>
where
    T: PositionKey + IsBitPacked,
{
    fn new(packed: BitPackedData<T>) -> Self {
        Self {
            pos: packed
        }
    }
    
    const fn new_const(packed: BitPackedData<T>) -> Self {
        Self {
            pos: packed
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_u32s() {
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::pack_val_raw(0u32, 0u32), 0);
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::pack_val_raw(0u32, 1u32), 1);
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::pack_val_raw(1u32, 0u32), 4294967296);
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::pack_val_raw(1u32, 1u32), 4294967297);
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::pack_val_raw(4294967295, 4294967295), 18446744073709551615);
    }

    #[test]
    fn unpack_u32s() {
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::unpack_raw(0u64), (0u32, 0u32));
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::unpack_raw(1u64), (0u32, 1u32));
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::unpack_raw(4294967296u64), (1u32, 0u32));
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::unpack_raw(4294967297u64), (1u32, 1u32));
        assert_eq!(<(u32, u32) as BitPackablePair<u64>>::unpack_raw(18446744073709551615u64), (4294967295u32, 4294967295u32));
    }
}
