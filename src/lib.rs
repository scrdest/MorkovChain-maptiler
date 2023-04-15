use crate::map::{FromArrayVec, MapNode, MapPosition, SquishyMapPosition, TileMap};
use crate::ruleset::GeneratorRuleset;

pub mod sampler;
pub mod map2d;
pub mod visualizers;
pub mod assigner;
pub mod mapgen_presets;
pub mod ruleset;
pub mod map;
mod map_node_state;
mod map_node_ordering;
mod position2d;
mod map2Dnode;

const COLORMAP_FILENAME: &str = "coloring_rules.json";
const RULESET_FILENAME: &str = "layout_rules.json";
const COMBINED_RULESET_FILENAME: &str = "rules.json";

fn generate<
    'a,
    MNa: MapNode<'a, 2, PositionKey=u8, Assignment=i8> + 'a,
    MNb: MapNode<'a, 2, PositionKey=u16, Assignment=i8> + 'a,
    MNc: MapNode<'a, 2, PositionKey=u32, Assignment=i8> + 'a,
    Ta: TileMap<'a, 2,  MNa>,
    Tb: TileMap<'a, 2,  MNb>,
    Tc: TileMap<'a, 2,  MNc>,
>(
    colormap_path: Option<&str>,
    rule_path: Option<&str>,
    map_size: Option<u32>
) where
    MNa::Position: SquishyMapPosition<'a, 2, 8, u8, u32>,
    MNb::Position: SquishyMapPosition<'a, 2, 8, u16, u32>,
    MNc::Position: SquishyMapPosition<'a, 2, 8, u32, u32>,
    <<MNa as MapNode<'a, 2>>::Position as FromArrayVec<2>>::Item: From<u8>,
    <<MNb as MapNode<'a, 2>>::Position as FromArrayVec<2>>::Item: From<u16>,
    <<MNc as MapNode<'a, 2>>::Position as FromArrayVec<2>>::Item: From<u32>,
    u8: From<<<MNa as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey>,
    u16: From<<<MNb as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey>,
    u32: From<<<MNc as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey>,
{
    let rules: GeneratorRuleset<i8> = GeneratorRuleset::from((
        colormap_path.unwrap_or(COLORMAP_FILENAME),
        rule_path.unwrap_or(RULESET_FILENAME),
        map_size
    ));
    rules.save(COMBINED_RULESET_FILENAME);
    match rules.map_size {
        0..=254 => rules.generate::<MNa::PositionKey, MNa, Ta>(),
        255..=65534 => rules.generate::<MNb::PositionKey, MNb, Tb>(),
        _ => rules.generate::<MNc::PositionKey, MNc, Tc>()
    };
}

pub fn generate_from_file<
    'a,
    MNa: MapNode<'a, 2, PositionKey=u8, Assignment=i8> + 'a,
    MNb: MapNode<'a, 2, PositionKey=u16, Assignment=i8> + 'a,
    MNc: MapNode<'a, 2, PositionKey=u32, Assignment=i8> + 'a,
    Ta: TileMap<'a, 2,  MNa>,
    Tb: TileMap<'a, 2,  MNb>,
    Tc: TileMap<'a, 2,  MNc>,
>(ruleset_file: Option<&str>)
where
    MNa::Position: SquishyMapPosition<'a, 2, 8, u8, u32>,
    MNb::Position: SquishyMapPosition<'a, 2, 8, u16, u32>,
    MNc::Position: SquishyMapPosition<'a, 2, 8, u32, u32>,
    <<MNa as MapNode<'a, 2>>::Position as FromArrayVec<2>>::Item: From<u8>,
    <<MNb as MapNode<'a, 2>>::Position as FromArrayVec<2>>::Item: From<u16>,
    <<MNc as MapNode<'a, 2>>::Position as FromArrayVec<2>>::Item: From<u32>,
    u8: From<<<MNa as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey>,
    u16: From<<<MNb as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey>,
    u32: From<<<MNc as MapNode<'a, 2>>::Position as MapPosition<'a, 2, 8>>::PositionKey>,
{
    let rules = GeneratorRuleset::load(
        ruleset_file.unwrap_or(COMBINED_RULESET_FILENAME)
    );
    match rules {
        Ok(ruleset) => match ruleset.map_size {
            0..=254 => ruleset.generate::<u8, MNa, Ta>(),
            255..=65534 => ruleset.generate::<u16, MNb, Tb>(),
            _ => ruleset.generate::<u32, MNc, Tc>()
        },
        Err(e) => {
            eprintln!("{}", e);
            generate::<MNa, MNb, MNc, Ta, Tb, Tc>(None, None, None)
        }
    };
}
