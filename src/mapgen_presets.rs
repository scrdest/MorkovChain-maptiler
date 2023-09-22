use std::fs::File;
use crate::assigner::MapColoringAssigner;
use crate::sampler::MultinomialDistribution;
use crate::types::GridMapDs;
use crate::types::PossiblyDirectedMultinomialDistribution::Undirected;
use super::visualizers::MapColor;

// pub(crate) const NULL_NAME: i8 = 0;
// pub(crate) const CORRIDOR: i8 = 1;
// pub(crate) const WALL: i8 = 2;
// pub(crate) const SPACE: i8 = 3;
// pub(crate) const DOOR: i8 = 4;

pub const TUNNELS_COLORMAP_FILENAME: &str = "morkovmap_colormap_tunnel.json";
pub const TUNNELS_RULESET_FILENAME: &str = "morkovmap_rules_tunnel.json";


// fn tunnels_generate_colormap() -> GridMapDs<i8, MapColor> {
//     let colormap: GridMapDs<i8, MapColor> = GridMapDs::from_iter([
//         (CORRIDOR, ril::Rgb::new(100, 110, 115).into()),
//         // (tunnel_name_mapping("entryway"), ril::Rgb::new(70, 180, 70).into()),
//         // (tunnel_name_mapping("room"), ril::Rgb::new(70, 160, 70).into()),
//         (WALL, ril::Rgb::new(50, 50, 60).into()),
//         // (tunnel_name_mapping("door"), ril::Rgb::new(190, 140, 70).into()),
//         (SPACE, ril::Rgb::new(5, 5, 10).into()),
//         // (tunnel_name_mapping("window"), ril::Rgb::new(50, 150, 200).into()),
//     ]);
//     colormap
// }
//
//
// fn tunnels_generate_rules() -> MapColoringAssigner<i8> {
//     let colormap = read_colormap(TUNNELS_COLORMAP_FILENAME);
//     let rules = GridMapDs::from([
//         // undecided
//         (NULL_NAME, MultinomialDistribution::uniform_over(colormap.keys().into_iter().map(|k| k.to_owned()))),
//         // corridor floor
//         (CORRIDOR, MultinomialDistribution::from(
//             GridMapDs::from([
//                 (CORRIDOR, 5.),
//                 // (tunnel_name_mapping("entryway"), 1.),
//                 // (tunnel_name_mapping("room"), 5.),
//                 (WALL, 3.),
//             ])
//         )),
//         // entryway floor
//         // (tunnel_name_mapping("entryway"), MultinomialDistribution::from(
//         //     GridMapDs::from([
//         //         (CORRIDOR, 10.),
//         //         (tunnel_name_mapping("entryway"), 1.),
//         //         (tunnel_name_mapping("room"), 1.),
//         //         (WALL, 2.),
//         //         (tunnel_name_mapping("door"), 50.),
//         //     ])
//         // )),
//         // inner floor
//         // (tunnel_name_mapping("room"), MultinomialDistribution::from(
//         //     GridMapDs::from([
//         //         (tunnel_name_mapping("entryway"), 10.),
//         //         (tunnel_name_mapping("room"), 1.),
//         //     ])
//         // )),
//         // walls
//         (WALL, MultinomialDistribution::from(
//             GridMapDs::from([
//                 (CORRIDOR, 15.),
//                 // (tunnel_name_mapping("room"), 0.000001),
//                 // (tunnel_name_mapping("entryway"), 0.000001),
//                 (WALL, 5.),
//                 (DOOR, 5.),
//                 (SPACE, 15.),
//             ])
//         )),
//         // door
//         (DOOR, MultinomialDistribution::from(
//             GridMapDs::from([
//                 (CORRIDOR, 8.),
//                 (WALL, 2.),
//                 // (tunnel_name_mapping("window"), 1.),
//             ])
//         )),
//         // space
//         (SPACE, MultinomialDistribution::from(
//             GridMapDs::from([
//                 (WALL, 1.),
//                 (SPACE, 20.),
//                 // (tunnel_name_mapping("window"), 1.),
//             ])
//         )),
//         // glass
//         // (tunnel_name_mapping("window"), MultinomialDistribution::from(
//         //     GridMapDs::from([
//         //         (CORRIDOR, 2.),
//         //         (WALL, 2.),
//         //         SPACE, 4.),
//         //         (tunnel_name_mapping("window"), 2.),
//         //     ])
//         // )),
//     ]);
//
//     MapColoringAssigner::with_rules(rules)
// }


pub(crate) const WATER: i8 = 1;
pub(crate) const GRASS: i8 = 2;
pub(crate) const SAND: i8 = 3;
pub(crate) const SNOW: i8 = 4;
pub(crate) const ROCKY: i8 = 5;

pub const LANDMASS_COLORMAP_FILENAME: &str = "morkovmap_colormap_landmass.json";
pub const LANDMASS_RULESET_FILENAME: &str = "morkovmap_rules_landmass.json";


fn landmass_generate_colormap() -> GridMapDs<i8, MapColor> {
    let colormap: GridMapDs<i8, MapColor> = GridMapDs::from_iter([
        (WATER, ril::Rgb::new(50, 50, 225).into()),
        (GRASS, ril::Rgb::new(50, 200, 50).into()),
        (SAND, ril::Rgb::new(200, 200, 50).into()),
        (SNOW, ril::Rgb::new(210, 210, 220).into()),
        (ROCKY, ril::Rgb::new(40, 40, 65).into()),
    ]);
    colormap
}


fn landmass_generate_rules() -> MapColoringAssigner<i8> {
    let rules = GridMapDs::from([
        // water
        (WATER, Undirected(MultinomialDistribution::from(
            GridMapDs::from([
                (WATER, 45.),
                (SAND, 1.),
                // (ROCKY, 0.000001),
            ])
        ))),
        // grass
        (GRASS, Undirected(MultinomialDistribution::from(
            GridMapDs::from([
                (GRASS, 80.),
                (SAND, 5.),
                (SNOW, 3.),
                (ROCKY, 1.),
            ]))
        )),
        // sand
        (SAND, Undirected(MultinomialDistribution::from(
            GridMapDs::from([
                (1, 50.),
                (GRASS, 40.),
                (SAND, 30.),
                (ROCKY, 0.000001),
            ])
        ))),
        // snow
        (SNOW, Undirected(MultinomialDistribution::from(
            GridMapDs::from([
                (GRASS, 10.),
                (SAND, 1.),
                (SNOW, 25.),
                (ROCKY, 5.),
            ])
        ))),
        // rocky
        (ROCKY, Undirected(MultinomialDistribution::from(
            GridMapDs::from([
                (1, 0.000001),
                (GRASS, 1.),
                (SAND, 0.000001),
                (SNOW, 45.),
                (ROCKY, 70.),
            ])
        ))),
    ]);

    MapColoringAssigner::with_rules(rules)
}


fn save_colormap(filepath: &str) -> GridMapDs<i8, MapColor> {
    let colormap = landmass_generate_colormap();
    let rule_file = File::create(filepath).unwrap();
    serde_json::to_writer_pretty(rule_file, &colormap).unwrap();
    colormap
}

pub fn read_colormap(filepath: &str) -> GridMapDs<i8, ril::Rgb> {
    let rule_file = File::open(filepath);
    let raw_result = match rule_file {
        Ok(rule_fh) => serde_json::from_reader(rule_fh).unwrap(),
        Err(err) => {
            eprintln!("{}", err);
            save_colormap(filepath)
        }
    };

    let cast_result = raw_result
        .iter()
        .map(|(k, v)| (
            k.to_owned(),
            { let rgb: ril::Rgb = v.to_owned().into(); rgb }
        )).collect();

    cast_result
}

pub fn save_rules(filepath: &str) -> MapColoringAssigner<i8> {
    let rules = landmass_generate_rules();
    serde_json::to_writer_pretty(
        File::create(filepath).unwrap(),
        &rules
    ).unwrap();
    rules
}

pub fn read_rules(filepath: &str) -> MapColoringAssigner<i8> {
    let rule_file = File::open(filepath);
    match rule_file {
        Ok(rule_fh) => serde_json::from_reader(rule_fh).unwrap(),
        Err(err) => {
            eprintln!("{}", err);
            save_rules(filepath)
        }
    }
}
