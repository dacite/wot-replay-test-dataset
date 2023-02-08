use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use wot_replay_parser::{ReplayError, ReplayParser};

pub fn main() -> Result<(), ReplayError> {
    let paths =
        utils::parse_dir("/home/dacite/Projects/wot-battle-results-parser/replays").unwrap();

    let map = Arc::new(DashMap::new());
    paths
        .par_iter()
        .for_each(|path| copy_replay(path.path(), map.clone()));

    Ok(())
}

fn copy_replay(path: PathBuf, map: Arc<DashMap<[u16; 4], i32>>) {
    println!("{}", path.display());
    let parser = ReplayParser::parse_file(&path).unwrap();

    // We only generate dataset for random battles
    if !is_random_battle(&parser) {
        return;
    }

    // This bit code was to be used to limit the number of replays collected per version
    // Now, it is just capped at 1 max.
    let mut value = map
        .entry(parser.parse_replay_version().unwrap())
        .or_insert(1);

    if *value == 0 {
        return;
    } else {
        *value -= 1
    }

    let replay_name = construct_replay_filename(&parser);
    let copy_path = format!("test/{}-{replay_name}.wotreplay", *value + 1);

    std::fs::copy(path, copy_path).unwrap();
}

fn is_random_battle(parser: &ReplayParser) -> bool {
    let json = parser.replay_json_start().unwrap();
    let battle_type: i32 = json.parse_value("/battleType").unwrap();

    return battle_type == 1;
}

fn construct_replay_filename(parser: &ReplayParser) -> String {
    let json = parser.replay_json_start().unwrap();

    let map: String = json.parse_value("/mapName").unwrap();
    let region: String = json.parse_value("/regionCode").unwrap();

    let version = parser.parse_replay_version().unwrap();
    let version = utils::version_as_string(version);

    let complete = if parser.replay_json_end().is_some() {
        "-full"
    } else {
        ""
    };

    format!("{version}-{map}-{region}{complete}")
}

trait ParseValue {
    fn parse_value<T: DeserializeOwned>(&self, path: &str) -> Option<T>;
}

impl ParseValue for serde_json::Value {
    fn parse_value<T: DeserializeOwned>(&self, path: &str) -> Option<T> {
        let value = self.pointer(path)?;

        serde_json::from_value(value.clone()).ok()
    }
}
