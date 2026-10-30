//! The release each item first appeared in. `since.tsv` is written by
//! `tools/generators/generate_doc_since.py` from the release tags; see `docs/tools/jacket-doc.md`.
use super::model::Module;
use std::collections::BTreeMap;

const DATA: &str = include_str!("since.tsv");

/// Marks the items of the oldest release the data covers: older than anything known.
const OLDER: &str = "-";

/// A version as numbers to compare, `None` for anything that is not `X.Y.Z`.
fn parse(version: &str) -> Option<(u32, u32, u32)> {
    let mut parts = version.split('.').map(|n| n.parse::<u32>().ok());
    let found = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(found)
}

struct Data {
    /// The releases the data covers, as `0.30.4`.
    covered: Vec<String>,
    /// Module, then item name, then the version it first appeared in.
    versions: BTreeMap<String, BTreeMap<String, String>>,
}

fn read(text: &str) -> Data {
    let mut covered = Vec::new();
    let mut versions: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("# covers:") {
            covered = rest
                .split_whitespace()
                .map(|t| t.trim_start_matches('v').to_owned())
                .collect();
        } else if !line.starts_with('#') {
            let mut cells = line.split('\t');
            if let (Some(module), Some(version), Some(names)) =
                (cells.next(), cells.next(), cells.next())
            {
                let entry = versions.entry(module.to_owned()).or_default();
                for name in names.split(' ') {
                    entry.insert(name.to_owned(), version.to_owned());
                }
            }
        }
    }
    Data {
        covered,
        versions,
    }
}

/// Sets `since` on every item and module. `built` is the release being documented: what the
/// data has no row for is new in it, unless it is a release the data already covers (then the
/// source is a development version and shows nothing). Items first seen in the oldest covered
/// release show nothing either.
pub fn assign(modules: &mut [Module], built: &str) {
    apply(&read(DATA), modules, built);
}

fn apply(data: &Data, modules: &mut [Module], built: &str) {
    let new = (!data.covered.iter().any(|c| c == built) && parse(built).is_some())
        .then(|| built.to_owned());
    for module in modules.iter_mut() {
        let known = data.versions.get(&module.id);
        let mut earliest: Option<(u32, u32, u32)> = None;
        let mut every_item_has_one = !module.items.is_empty();
        for item in &mut module.items {
            let version = match known.and_then(|k| k.get(&item.name)) {
                Some(v) if v == OLDER => None,
                Some(v) => Some(v.clone()),
                None => new.clone(),
            };
            match version.as_deref().and_then(parse) {
                Some(v) => earliest = Some(earliest.map_or(v, |e| e.min(v))),
                None => every_item_has_one = false,
            }
            item.since = version.unwrap_or_default();
        }
        module.since = if every_item_has_one {
            earliest
                .map(|(a, b, c)| format!("{a}.{b}.{c}"))
                .unwrap_or_default()
        } else {
            String::new()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::model::{Item, Kind};

    fn module(id: &str, names: &[&str]) -> Module {
        Module {
            id: id.into(),
            file: String::new(),
            docs: String::new(),
            docs_line: 0,
            platforms: Vec::new(),
            items: names
                .iter()
                .map(|n| Item {
                    name: (*n).into(),
                    kind: Kind::Constant,
                    file: String::new(),
                    line: 0,
                    docs: String::new(),
                    value: String::new(),
                    overloads: Vec::new(),
                    members: Vec::new(),
                    group: String::new(),
                    platforms: Vec::new(),
                    since: String::new(),
                })
                .collect(),
            category: String::new(),
            wasm: false,
            since: String::new(),
        }
    }

    const SAMPLE: &str = "# covers: v0.1.0 v0.2.0\nA\t-\told\nA\t0.2.0\tnewer\nB\t0.2.0\tfresh\n";

    #[test]
    fn items_get_the_release_that_added_them() {
        let data = read(SAMPLE);
        let mut mods = [module("A", &["old", "newer"]), module("B", &["fresh"])];
        apply(&data, &mut mods, "0.2.0");
        assert_eq!(mods[0].items[0].since, "");
        assert_eq!(mods[0].items[1].since, "0.2.0");
        assert_eq!(mods[0].since, "");
        assert_eq!(mods[1].since, "0.2.0");
    }

    #[test]
    fn what_the_data_lacks_is_new_in_an_uncovered_release_only() {
        let data = read(SAMPLE);
        let mut mods = [module("A", &["old", "added"]), module("C", &["x"])];
        apply(&data, &mut mods, "0.3.0");
        assert_eq!(mods[0].items[1].since, "0.3.0");
        assert_eq!(mods[1].since, "0.3.0");
        apply(&data, &mut mods, "0.2.0");
        assert_eq!(mods[0].items[1].since, "");
        assert_eq!(mods[1].since, "");
    }

    #[test]
    fn the_checked_in_data_reads() {
        let data = read(DATA);
        assert!(data.covered.len() > 1);
        assert!(data.versions.contains_key("String"));
    }
}
