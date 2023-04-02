use std::borrow::Borrow;
use std::collections::HashMap;
use num::NumCast;
use crate::map2d::{Map2D, PositionKey};
use crate::sampler::DistributionKey;
use ril;
use ril::{Draw, Rgb};
use serde::{Serialize, Deserialize};


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


pub trait MapVisualizer<N: DistributionKey, P: PositionKey> {
    type Output;

    fn visualise<'v>(&self, map: &'v Map2D<N, P>) -> Option<Self::Output>;
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

impl<N: DistributionKey, P: PositionKey + NumCast + Into<u32>> MapVisualizer<N, P> for RilPixelVisualizer<N> {
    type Output = ();

    fn visualise(&self, map: &Map2D<N, P>) -> Option<Self::Output> {
        const MAP_SCALE_FACTOR: u32 = 1;
        let xspan_raw = P::one() + map.max_pos.x - map.min_pos.x;
        let yspan_raw = P::one() + map.max_pos.y - map.min_pos.y;

        // ASSUMPTION: NumCast will return None only for *smaller* datatypes
        // (64s should get converted with truncation, i32 by dropping the sign)
        if xspan_raw >= P::from(u32::MAX).unwrap_or(P::max_value()) {
            println!("WARNING: map X-span too large, map image will be truncated!")
        }

        if yspan_raw >= P::from(u32::MAX).unwrap_or(P::max_value()) {
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
            let tilereader = tile.read().unwrap();
            if tilereader.assignment.is_none() { continue; }
            let tilepos = tilereader.position;

            let tilepos_x_relative = (tilepos.x - map.min_pos.x) * P::from(MAP_SCALE_FACTOR).unwrap();
            let tilepos_y_relative = (tilepos.y - map.min_pos.y) * P::from(MAP_SCALE_FACTOR).unwrap();

            if tilepos_x_relative > P::from(u32::MAX).unwrap_or(P::max_value()) {continue}
            if tilepos_y_relative > P::from(u32::MAX).unwrap_or(P::max_value()) {continue}

            let tilepos_x_relative_cast: u32 = tilepos_x_relative.into();
            let tilepos_y_relative_cast: u32 = tilepos_y_relative.into();

            let fillcolor = self.color_lookup.get(
                // We can unwrap here since we checked earlier it's not None.
                &tilereader.assignment.unwrap()
            ).map(|mc|
                // Convert to Rgb...
                mc.to_owned().into()
            ).unwrap_or(
                // ...or default if we have no color spec.
                Rgb::white()
            );

            //println!("{:?} => {:?}", tilepos, &tilereader.assignment);

            let repr: ril::Rectangle<ril::Rgb> = ril::Rectangle::new()
                .with_size(MAP_SCALE_FACTOR, MAP_SCALE_FACTOR)
                .with_fill(fillcolor)
                .with_position(tilepos_x_relative_cast, tilepos_y_relative_cast)
            ;
            repr.draw(&mut image);
        }

        let result = image.save_inferred("map.png");
        match result {
            Ok(_) => Some(()),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        }
    }
}
