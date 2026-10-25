//! `jacket.toml` loading on arbitrary text.
//!
//! The input is the text of a `jacket.toml`. Every input must load without panicking (a bad file
//! is an error), loading agrees with parsing, and settings serialize and read back to the same
//! settings.
use jacket_config::{Config, Files, load, parse};
use std::path::Path;

/// One `/p/jacket.toml` in memory.
struct One(String);

impl Files for One {
    fn is_file(&self, path: &Path) -> bool {
        path == Path::new("/p/jacket.toml")
    }

    fn read(&self, path: &Path) -> std::io::Result<String> {
        if self.is_file(path) {
            Ok(self.0.clone())
        } else {
            Err(std::io::ErrorKind::NotFound.into())
        }
    }
}

pub fn jacket_config(data: &[u8]) {
    let text = String::from_utf8_lossy(data).into_owned();
    let loaded = load(&One(text.clone()), Path::new("/p"));
    match (parse(&text), loaded) {
        (Ok(config), Ok(Some(loaded))) => {
            assert_eq!(config, loaded.config);
            round_trip(&config);
        }
        (Err(_), Err(_)) => {}
        (parsed, loaded) => panic!("jacket.toml read two ways: {parsed:?} / {loaded:?}"),
    }
}

/// Written out and read back, the settings are the same.
fn round_trip(config: &Config) {
    let written = toml::to_string(config)
        .unwrap_or_else(|e| panic!("settings do not serialize: {e}\n{config:?}"));
    let again =
        parse(&written).unwrap_or_else(|e| panic!("settings do not read back: {e}\n{written}"));
    let twice = toml::to_string(&again).expect("serializes again");
    assert_eq!(written, twice, "serialization is not stable");
}
