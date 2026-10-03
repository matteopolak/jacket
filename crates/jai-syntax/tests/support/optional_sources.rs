//! Original source inventories are local evidence, not tracked CI dependencies.
use std::{fs, io::ErrorKind, path::Path};

pub fn read_optional(relative: &str) -> Option<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            eprintln!(
                "SKIP optional original-source probe: {} is absent",
                path.display()
            );
            return None;
        }
        Err(error) => panic!(
            "cannot read optional original source {}: {error}",
            path.display()
        ),
    };
    Some(
        jai_lexer::decode_source(&bytes)
            .unwrap_or_else(|error| {
                panic!(
                    "cannot decode optional original source {}: {error}",
                    path.display()
                )
            })
            .into_owned(),
    )
}
