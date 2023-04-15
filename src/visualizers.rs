use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::RwLockReadGuard;
use ril;
use ril::{Draw, Rgb};
use serde::{Deserialize, Serialize};
use crate::sampler::DistributionKey;
use crate::map::{BoundedMapPosition, MapNode, MapPosition, PositionKey, SquishyMapPosition, TileMap};
use crate::map_node_state::MapNodeState;


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


pub trait MapVisualizer<'a, MN: MapNode<'a, 2> + 'a, M: TileMap<'a, 2, MN>> {
    type Output;

    fn visualise(&self, map: &M) -> Option<Self::Output>;
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

impl<K: DistributionKey> From<HashMap<K, MapColor>> for RilPixelVisualizer<K> {
    fn from(value: HashMap<K, MapColor>) -> Self {
        Self::new(value)
    }
}

impl<'a, P, MN: MapNode<'a, 2, PositionKey=P> + 'a, M: TileMap<'a, 2, MN>> MapVisualizer<'a, MN, M> for RilPixelVisualizer<MN::Assignment>
where
    MN::Position: SquishyMapPosition<'a, 2, 8, P, u32>,
    P: PositionKey + Into<MN::PositionKey>
{
    type Output = ();

    fn visualise(&self, map: &M) -> Option<Self::Output> {
        // const MAP_SCALE_FACTOR: MN = 1;

        let pos_unit = P::one();
        // let pos_scale = P::from(MAP_SCALE_FACTOR);

        let min_pos_position = map.get_min_pos();
        let max_pos_position = map.get_max_pos();

        let min_pos_x_raw = min_pos_position.get_dim(0).unwrap();
        let max_pos_x_raw = max_pos_position.get_dim(0).unwrap();

        let min_pos_y_raw = min_pos_position.get_dim(1).unwrap();
        let max_pos_y_raw = max_pos_position.get_dim(1).unwrap();

        // let min_pos_x = min_pos_x_raw;
        // let max_pos_x = max_pos_x_raw;
        //
        // let min_pos_y = min_pos_y_raw;
        // let max_pos_y = max_pos_y_raw;

        // let xspan_raw = pos_unit + max_pos_x - min_pos_x;
        // let yspan_raw = pos_unit + max_pos_y - min_pos_y;

        let trg_span = u32::MAX - u32::MIN;

        if MN::Position::any_out_of_bounds([min_pos_x_raw, max_pos_x_raw]) {
            println!("WARNING: map X-span too large, map image will be truncated!")
        };

        if MN::Position::any_out_of_bounds([min_pos_y_raw, max_pos_y_raw]) {
            println!("WARNING: map Y-span too large, map image will be truncated!")
        };

        let squish_min = min_pos_position.squish();
        let squish_max = max_pos_position.squish();

        let xspan: u32 = squish_max.get_dim(0).unwrap() - squish_min.get_dim(0).unwrap();
        let yspan: u32 = squish_max.get_dim(1).unwrap() - squish_min.get_dim(1).unwrap();

        let mut image = ril::Image::new(
            xspan,
            yspan,
            ril::Rgb::new(255, 200, 50)
        );

        let tile_iter = map.get_tiles().into_iter();

        for tile in tile_iter {
            let tilereader: RwLockReadGuard<MN> = tile.read().unwrap();

            let assignment = match &tilereader.get_state() {
                MapNodeState::Finalized(asgn) => asgn,
                MapNodeState::Undecided(_) => continue
            };
            let tilepos = tilereader.get_position().squish();
            let squished_pos_x = tilepos.get_dim(0).unwrap();
            let squished_pos_y = tilepos.get_dim(1).unwrap();

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
                .with_size(1u32, 1u32)
                .with_fill(fillcolor)
                .with_position(u32::from(squished_pos_x), u32::from(squished_pos_y))
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
