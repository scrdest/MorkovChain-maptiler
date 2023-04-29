use crate::adjacency::{OctileAdjacencyGenerator, CardinalAdjacencyGenerator};
use crate::position2d::Position2D;
use crate::ruleset::GeneratorRuleset;
use crate::sampler::DistributionKey;

pub mod sampler;
pub mod map2d;
pub mod visualizers;
pub mod assigner;
pub mod mapgen_presets;
pub mod ruleset;
pub mod map;
pub mod position;
pub mod position2d;
pub mod map2dnode;
pub mod adjacency;

const COLORMAP_FILENAME: &str = "coloring_rules.json";
const RULESET_FILENAME: &str = "layout_rules.json";
const COMBINED_RULESET_FILENAME: &str = "rules.json";

fn generate(colormap_path: Option<&str>, rule_path: Option<&str>, map_size: Option<u32>) {
    let rules: GeneratorRuleset<i8> = GeneratorRuleset::from((
        colormap_path.unwrap_or(COLORMAP_FILENAME),
        rule_path.unwrap_or(RULESET_FILENAME),
        map_size
    ));
    rules.save(COMBINED_RULESET_FILENAME);
    match rules.map_size {
        0..=254 => rules.generate::<OctileAdjacencyGenerator<Position2D<u8>>, Position2D<u8>>(),
        255..=65534 => rules.generate::<OctileAdjacencyGenerator<Position2D<u16>>, Position2D<u16>>(),
        _ => rules.generate::<OctileAdjacencyGenerator<Position2D<u32>>, Position2D<u32>>()
    };
}

fn generate_from_ruleset<T: DistributionKey>(ruleset: &GeneratorRuleset<T>) {
    let normalized_adjacency = ruleset.adjacency.as_ref().map(
        |s| s.as_str().trim().to_lowercase()
    ).unwrap_or_default();

    match normalized_adjacency.as_str() {
        "cardinal" => match ruleset.map_size {
            0..=254 => ruleset.generate::<CardinalAdjacencyGenerator<Position2D<u8>>, Position2D<u8>>(),
            255..=65534 => ruleset.generate::<CardinalAdjacencyGenerator<Position2D<u16>>, Position2D<u16>>(),
            _ => ruleset.generate::<CardinalAdjacencyGenerator<Position2D<u32>>, Position2D<u32>>()
        },
        "octile" | _ => match ruleset.map_size {
            0..=254 => ruleset.generate::<OctileAdjacencyGenerator<Position2D<u8>>, Position2D<u8>>(),
            255..=65534 => ruleset.generate::<OctileAdjacencyGenerator<Position2D<u16>>, Position2D<u16>>(),
            _ => ruleset.generate::<OctileAdjacencyGenerator<Position2D<u32>>, Position2D<u32>>()
        },
    }

}

pub fn generate_from_file(ruleset_file: Option<&str>) {
    let rules = GeneratorRuleset::load(
        ruleset_file.unwrap_or(COMBINED_RULESET_FILENAME)
    );
    match rules {
        Ok(ruleset) => generate_from_ruleset(&ruleset),
        Err(e) => {
            eprintln!("{}", e);
            generate(None, None, None)
        }
    };
}
