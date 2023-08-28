use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use num::{Bounded, NumCast, One};
use crate::map2d::Map2D;
use crate::sampler::DistributionKey;
use ril;
use ril::{Draw, Rgb};
use serde::{Deserialize, Serialize};
use crate::adjacency::AdjacencyGenerator;
use crate::map2dnode::MapNodeState;
use crate::position::{MapPosition, PositionKey};


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MapColor {
    Rgb(u8, u8, u8)
}

impl Into<ril::Rgb> for MapColor {
    fn into(self) -> Rgb {
        match self {
            Self::Rgb(r, g, b) => ril::Rgb::new(r, g, b)
        }
    }
}

impl<R: Borrow<ril::Rgb>> From<R> for MapColor {
    fn from(value: R) -> Self {
        Self::Rgb(value.borrow().r, value.borrow().g, value.borrow().b)
    }
}


pub trait MapVisualizer<AG: AdjacencyGenerator<2>, N: DistributionKey, MP: MapPosition<2>> {
    type Output;
    type Args;

    fn visualise(&self, map: &Map2D<AG, N, MP>, args: Option<Self::Args>) -> Option<Self::Output>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RilPixelVisualizer<N: DistributionKey> {
    color_lookup: HashMap<N, MapColor>
}

impl<N: DistributionKey> RilPixelVisualizer<N> {
    pub fn new(color_lookup: HashMap<N, MapColor>) -> Self {
        Self {
            color_lookup
        }
    }
}

impl<N: DistributionKey> From<&HashMap<N, ril::Rgb>> for RilPixelVisualizer<N> {
    fn from(value: &HashMap<N, ril::Rgb>) -> Self {
        Self::new(value
            .iter()
            .map(|(k, v)| (
                k.to_owned(),
                MapColor::from(v.to_owned())
            )).collect()
        )
    }
}

impl<N: DistributionKey> From<HashMap<N, MapColor>> for RilPixelVisualizer<N> {
    fn from(value: HashMap<N, MapColor>) -> Self {
        Self::new(value)
    }
}

impl<AG: AdjacencyGenerator<2>, N: DistributionKey, MP: MapPosition<2>> MapVisualizer<AG, N, MP> for RilPixelVisualizer<N>
where MP::Key: PositionKey + NumCast + Into<u32>
{
    type Output = ();
    type Args = String;

    fn visualise(&self, map: &Map2D<AG, N, MP>, output: Option<Self::Args>) -> Option<Self::Output> {
        const MAP_SCALE_FACTOR: u32 = 1;

        let min_pos = map.min_pos.get_dims();
        let max_pos = map.max_pos.get_dims();

        let xspan_raw = MP::Key::one() + max_pos[0] - min_pos[0];
        let yspan_raw = MP::Key::one() + max_pos[1] - min_pos[1];

        // ASSUMPTION: NumCast will return None only for *smaller* datatypes
        // (64s should get converted with truncation, i32 by dropping the sign)
        if xspan_raw >= <<MP as MapPosition<2>>::Key as NumCast>::from(u32::MAX).unwrap_or(MP::Key::max_value()) {
            println!("WARNING: map X-span too large, map image will be truncated!")
        }

        if yspan_raw >= <<MP as MapPosition<2>>::Key as NumCast>::from(u32::MAX).unwrap_or(MP::Key::max_value()) {
            println!("WARNING: map Y-span too large, map image will be truncated!")
        }

        let xspan: u32 = xspan_raw.into();
        let yspan: u32 = yspan_raw.into();

        let mut image = ril::Image::new(
            xspan * MAP_SCALE_FACTOR,
            yspan * MAP_SCALE_FACTOR,
            ril::Rgb::new(255, 200, 50)
        );

        for tile in &map.tiles {
            let tilereader = tile.try_borrow().unwrap();
            let assignment = match &tilereader.state {
                MapNodeState::Finalized(asgn) => asgn,
                MapNodeState::Undecided(_) => continue
            };
            let tilepos = tilereader.position.get_dims();

            let tilepos_x_relative = (
                tilepos[0] - min_pos[0]
            ) * <<MP as MapPosition<2>>::Key as NumCast>::from(MAP_SCALE_FACTOR).unwrap();

            let tilepos_y_relative = (
                tilepos[1] - min_pos[1]
            ) * <<MP as MapPosition<2>>::Key as NumCast>::from(MAP_SCALE_FACTOR).unwrap();

            if tilepos_x_relative > <<MP as MapPosition<2>>::Key as NumCast>::from(u32::MAX).unwrap_or(MP::Key::max_value()) {continue}
            if tilepos_y_relative > <<MP as MapPosition<2>>::Key as NumCast>::from(u32::MAX).unwrap_or(MP::Key::max_value()) {continue}

            let tilepos_x_relative_cast: u32 = tilepos_x_relative.into();
            let tilepos_y_relative_cast: u32 = tilepos_y_relative.into();

            let fillcolor = self.color_lookup
                .get(assignment)
                .map(|mc|
                    // Convert to Rgb...
                    mc.to_owned().into()
                ).unwrap_or(
                // ...or default if we have no color spec.
                Rgb::white()
            );

            // println!("{:?} => {:?}", tilepos, assignment);

            let repr: ril::Rectangle<ril::Rgb> = ril::Rectangle::new()
                .with_size(MAP_SCALE_FACTOR, MAP_SCALE_FACTOR)
                .with_fill(fillcolor)
                .with_position(tilepos_x_relative_cast, tilepos_y_relative_cast)
            ;
            repr.draw(&mut image);
        }

        let fname = output.unwrap_or(Self::Args::from("map.png"));
        let result = image.save_inferred(fname);
        match result {
            Ok(_) => Some(()),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        }
    }
}
