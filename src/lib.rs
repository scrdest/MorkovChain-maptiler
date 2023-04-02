use crate::ruleset::GeneratorRuleset;

pub mod sampler;
pub mod map2d;
pub mod visualizers;
pub mod assigner;
pub mod mapgen_presets;
pub mod ruleset;
pub mod map;

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
        0..=254 => rules.generate::<u8>(),
        255..=65534 => rules.generate::<u16>(),
        _ => rules.generate::<u32>()
    };
}

pub fn generate_from_file(ruleset_file: Option<&str>) {
    let rules = GeneratorRuleset::load(
        ruleset_file.unwrap_or(COMBINED_RULESET_FILENAME)
    );
    match rules {
        Ok(ruleset) => match ruleset.map_size {
            0..=254 => ruleset.generate::<u8>(),
            255..=65534 => ruleset.generate::<u16>(),
            _ => ruleset.generate::<u32>()
        },
        Err(e) => {
            eprintln!("{}", e);
            generate(None, None, None)
        }
    };
}
