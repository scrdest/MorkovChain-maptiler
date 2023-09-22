use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{Debug};
use std::fs::File;
use num::{Bounded, NumCast, One};
use crate::map2d::Map2D;
use crate::sampler::DistributionKey;
use ril;
use ril::{Draw, Rgb};
use serde::{Deserialize, Serialize};
use crate::adjacency::AdjacencyGenerator;
use crate::map2dnode::{Map2DNode, MapNodeState};
use crate::position::{ConvertibleMapPosition, MapPosition, PositionKey};
use crate::position2d::CompactMapPosition;
use crate::types::GridMapDs;


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
    color_lookup: GridMapDs<N, MapColor>
}

impl<N: DistributionKey> RilPixelVisualizer<N> {
    pub fn new(color_lookup: GridMapDs<N, MapColor>) -> Self {
        Self {
            color_lookup
        }
    }
}

impl<N: DistributionKey> From<&GridMapDs<N, ril::Rgb>> for RilPixelVisualizer<N> {
    fn from(value: &GridMapDs<N, ril::Rgb>) -> Self {
        Self::new(value
            .iter()
            .map(|(k, v)| (
                k.to_owned(),
                MapColor::from(v.to_owned())
            )).collect()
        )
    }
}

impl<N: DistributionKey> From<GridMapDs<N, MapColor>> for RilPixelVisualizer<N> {
    fn from(value: GridMapDs<N, MapColor>) -> Self {
        Self::new(value)
    }
}

impl<AG: AdjacencyGenerator<2>, DK: DistributionKey, MP: MapPosition<2>> MapVisualizer<AG, DK, MP> for RilPixelVisualizer<DK>
where MP::Key: PositionKey + NumCast + Into<u32>
{
    type Output = ();
    type Args = String;

    fn visualise(&self, map: &Map2D<AG, DK, MP>, output: Option<Self::Args>) -> Option<Self::Output> {
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

#[derive(Debug, Clone)]
pub struct JsonDataVisualizer {}

impl JsonDataVisualizer {
    pub fn new() -> Self {
        Self {

        }
    }
}


#[derive(Debug, Clone)]
pub struct ImageAndJsonDataVisualizer<DK: DistributionKey + Serialize> {
    img: RilPixelVisualizer<DK>,
    json: JsonDataVisualizer
}

impl<DK: DistributionKey + Serialize> ImageAndJsonDataVisualizer<DK> {
    pub fn new(img: RilPixelVisualizer<DK>, json: JsonDataVisualizer) -> Self {
        Self {
            img, json
        }
    }
}

impl<DK: DistributionKey + Serialize> From<(RilPixelVisualizer<DK>, JsonDataVisualizer)> for ImageAndJsonDataVisualizer<DK> {
    fn from(value: (RilPixelVisualizer<DK>, JsonDataVisualizer)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl<DK: DistributionKey + Serialize> From<(JsonDataVisualizer, RilPixelVisualizer<DK>)> for ImageAndJsonDataVisualizer<DK> {
    fn from(value: (JsonDataVisualizer, RilPixelVisualizer<DK>)) -> Self {
        Self::new(value.1, value.0)
    }
}

impl<AG: AdjacencyGenerator<2>, PK, MP: MapPosition<2>> MapVisualizer<AG, PK, MP> for ImageAndJsonDataVisualizer<PK>
where
    MP::Key: NumCast + Serialize,
    PK: PositionKey + NumCast + Into<u32> + Serialize + Default,
    MP: MapPosition<2, Key=PK> + ConvertibleMapPosition<2, PK, CompactMapPosition<PK>>
{
    type Output = (<RilPixelVisualizer<PK> as MapVisualizer<AG, PK, MP>>::Output, <JsonDataVisualizer as MapVisualizer<AG, PK, MP>>::Output);
    type Args = (<RilPixelVisualizer<PK> as MapVisualizer<AG, PK, MP>>::Args, <JsonDataVisualizer as MapVisualizer<AG, PK, MP>>::Args);

    fn visualise(&self, map: &Map2D<AG, PK, MP>, args: Option<Self::Args>) -> Option<Self::Output> {
        let (left_args, right_args) = match args {
            None => (None, None),
            Some(tup) => (Some(tup.0), Some(tup.1))
        };
        self.img.visualise(map, left_args);
        self.json.visualise(map, right_args);
        Some(((), ()))
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Map2DNodeSerialized<K: DistributionKey, MP: MapPosition<2>> {
    #[serde(rename(serialize = "p", deserialize = "position"))]
    pub(crate) position: MP,

    #[serde(rename(serialize = "s", deserialize = "state"))]
    pub(crate) state: K,
}

// #[derive(Clone, Serialize, Deserialize)]
// #[serde(transparent)]
// pub struct Map2DNodeSerializedStr {
//     pub(crate) data: String
// }

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct SerializableMap2D<
    K: DistributionKey + Serialize,
    MP: MapPosition<2> + Serialize
> {
    pub tiles: Vec<Map2DNodeSerialized<K, MP>>,
    // position_index: GridMapDs<MP, Map2DNodeSerialized<K, MP>>,
    // pub(crate) min_pos: MP,
    // pub(crate) max_pos: MP,
}

impl<
    AG: AdjacencyGenerator<2>,
    K: DistributionKey + Serialize,
    T: PositionKey + Serialize,
    MP: MapPosition<2, Key=T> + ConvertibleMapPosition<2, T, CompactMapPosition<T>>
> From<&Map2D<AG, K, MP>> for SerializableMap2D<K, CompactMapPosition<T>>
{
    fn from(value: &Map2D<AG, K, MP>) -> Self {
        let nodes = value.tiles.iter().map(
            |raw_tile| {
                let a: &RefCell<Map2DNode<AG, K, MP>> = raw_tile.borrow();
                let owned_tile = a.to_owned().into_inner();

                let state = match owned_tile.state {
                    MapNodeState::Finalized(stateval) => stateval,
                    _ => panic!("Undecided tiles left!")
                };

                let position = owned_tile.position.convert();

                Map2DNodeSerialized {
                    position,
                    state
                }
            }
        );

        let size_estimate = value.tiles.len();
        let mut tile_vec: Vec<Map2DNodeSerialized<K, CompactMapPosition<MP::Key>>> = Vec::with_capacity(size_estimate);
        // let mut position_hashmap: GridMapDs<MP, Map2DNodeSerialized<K, MP>> = GridMapDs::with_capacity(size_estimate);

        nodes.for_each(|node| {
            tile_vec.push(node);
            // position_hashmap.insert(node.position.to_owned(), node);
        });

        Self {
            tiles: tile_vec,
            // position_index: position_hashmap,
            // min_pos: value.min_pos,
            // max_pos: value.max_pos
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct CompactedSerializableMap2D<const SEP: char> {
    pub tiles: Vec<String>
}

impl<
    const SEP: char,
    AG: AdjacencyGenerator<2>,
    K: DistributionKey + Serialize,
    T: PositionKey + Serialize,
    MP: MapPosition<2, Key=T> + ConvertibleMapPosition<2, T, CompactMapPosition<T>>
> From<&Map2D<AG, K, MP>> for CompactedSerializableMap2D<SEP>
{
    fn from(value: &Map2D<AG, K, MP>) -> Self {
        let nodes = value.tiles.iter().map(
            |raw_tile| {
                let a: &RefCell<Map2DNode<AG, K, MP>> = raw_tile.borrow();
                let owned_tile = a.to_owned().into_inner();

                let state = match owned_tile.state {
                    MapNodeState::Finalized(stateval) => stateval,
                    MapNodeState::Undecided(dist) => {
                        println!("Undecided tiles left at {:?}!", owned_tile.position.get_dims());
                        dist.sample_with_default_rng()
                    }
                    // _ => panic!("{}", format!("Undecided tiles left at {:?}!", owned_tile.position))
                };

                let position = owned_tile.position.convert();


                Map2DNodeSerialized {
                    position,
                    state
                }
            }
        );

        let strings = nodes.map(
            |node| {
                let dims = node.position.get_dims().map(
                    |dim| serde_json::to_string(&dim).unwrap()
                );
                let state = serde_json::to_string(&node.state).unwrap();

                let str_node = format!("{s}{sep}{x},{y}", s=state, sep=SEP, x=dims[0], y=dims[1]);
                str_node
            }
        );

        let size_estimate = value.tiles.len();
        let mut tile_vec: Vec<String> = Vec::with_capacity(size_estimate);
        // let mut position_hashmap: GridMapDs<MP, Map2DNodeSerialized<K, MP>> = GridMapDs::with_capacity(size_estimate);

        strings.for_each(|node| {
            tile_vec.push(node);
            // position_hashmap.insert(node.position.to_owned(), node);
        });

        Self {
            tiles: tile_vec
        }
    }
}


impl<AG: AdjacencyGenerator<2>, DK: DistributionKey + Serialize, PK, MP>
MapVisualizer<AG, DK, MP> for JsonDataVisualizer
where
    PK: PositionKey + NumCast + Into<u32> + Serialize,
    MP: MapPosition<2, Key=PK> + ConvertibleMapPosition<2, PK, CompactMapPosition<PK>>
{
    type Output = ();
    type Args = String;

    fn visualise(&self, map: &Map2D<AG, DK, MP>, output: Option<Self::Args>) -> Option<Self::Output> {
        let fname = output.unwrap_or(Self::Args::from("genmap.json"));

        let castmap: CompactedSerializableMap2D<'|'> = CompactedSerializableMap2D::from(map);

        let result = serde_json::to_writer(
            File::create(fname).unwrap(),
            &castmap.tiles
        );

        match result {
            Ok(_) => Some(()),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        }
    }
}

