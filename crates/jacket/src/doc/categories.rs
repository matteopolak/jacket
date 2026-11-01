//! Where each module is listed: under which `[[doc.category]]` heading and in which
//! `[[doc.group]]`, both matched against the module's folder by the patterns of `jacket.toml`.
use jacket_config::pattern::PathPattern;
use jacket_config::schema::{DocCategory, DocGroup};

/// A heading modules are listed under, with the badge they and their items carry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Category {
    pub name: String,
    /// Badge text; empty for none.
    pub badge: String,
    pub tip: String,
    pub color: String,
    pub color_dark: String,
}

/// Where each module goes, in the order of the ids given.
pub struct Layout {
    pub categories: Vec<Category>,
    /// Group names, in display order.
    pub groups: Vec<String>,
    /// For each module: its category's index, and its group (empty for none).
    pub of: Vec<(Option<usize>, String)>,
}

/// Whether `color` is safe to put in a stylesheet: a `#` hex colour or a plain name.
fn plain_color(color: &str) -> bool {
    match color.strip_prefix('#') {
        Some(hex) => {
            matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit())
        }
        None => !color.is_empty() && color.chars().all(|c| c.is_ascii_alphabetic()),
    }
}

/// The index of the last entry whose patterns match `folder`.
fn last_match<'a>(entries: impl Iterator<Item = &'a [PathPattern]>, folder: &str) -> Option<usize> {
    let mut found = None;
    for (i, patterns) in entries.enumerate() {
        if patterns.iter().any(|p| p.is_match(folder)) {
            found = Some(i);
        }
    }
    found
}

/// Lays out the modules whose folders (relative to the `jacket.toml`, `/`-separated) are
/// `folders`, named `ids` in messages.
pub fn layout(
    categories: &[DocCategory],
    groups: &[DocGroup],
    ids: &[String],
    folders: &[String],
) -> Result<Layout, String> {
    let mut problems = Vec::new();
    let mut out = Vec::with_capacity(categories.len());
    for c in categories {
        let mut color = c.color.clone().unwrap_or_default();
        let mut color_dark = c.color_dark.clone().unwrap_or_else(|| color.clone());
        for (key, value) in [("color", &mut color), ("color_dark", &mut color_dark)] {
            if !value.is_empty() && !plain_color(value) {
                problems.push(format!(
                    "category `{}`: `{key}` `{value}` is not a `#rrggbb` colour or a colour name",
                    c.name
                ));
                value.clear();
            }
        }
        out.push(Category {
            name: c.name.clone(),
            badge: c.badge.clone().unwrap_or_default(),
            tip: c.tip.clone().unwrap_or_else(|| c.name.clone()),
            color,
            color_dark,
        });
    }
    let mut names: Vec<String> = Vec::new();
    for g in groups {
        if names.contains(&g.name) {
            problems.push(format!("group `{}` is declared twice", g.name));
        } else {
            names.push(g.name.clone());
        }
    }
    let unused = |kind: &str, name: &str, patterns: &[PathPattern]| {
        patterns
            .iter()
            .filter(|p| !folders.iter().any(|f| p.is_match(f)))
            .map(|p| format!("`{p}` (in {kind} `{name}`) matches no module"))
            .collect::<Vec<_>>()
    };
    for c in categories {
        problems.extend(unused("category", &c.name, &c.modules));
    }
    for g in groups {
        problems.extend(unused("group", &g.name, &g.modules));
    }
    let mut of = Vec::with_capacity(folders.len());
    for (id, folder) in ids.iter().zip(folders) {
        let category = last_match(categories.iter().map(|c| c.modules.as_slice()), folder);
        let group = last_match(groups.iter().map(|g| g.modules.as_slice()), folder);
        let group = match group {
            Some(g) => groups[g].name.clone(),
            None if groups.is_empty() => String::new(),
            None => {
                problems.push(format!("module `{id}` ({folder}) is in no group"));
                String::new()
            }
        };
        of.push((category, group));
    }
    if problems.is_empty() {
        Ok(Layout {
            categories: out,
            groups: names,
            of,
        })
    } else {
        Err(problems.join("\n  "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| (*n).to_owned()).collect()
    }

    fn patterns(p: &[&str]) -> Vec<PathPattern> {
        p.iter().map(|p| PathPattern::new(p).unwrap()).collect()
    }

    fn category(name: &str, p: &[&str]) -> DocCategory {
        DocCategory {
            name: name.into(),
            modules: patterns(p),
            ..DocCategory::default()
        }
    }

    fn group(name: &str, p: &[&str]) -> DocGroup {
        DocGroup {
            name: name.into(),
            modules: patterns(p),
        }
    }

    #[test]
    fn the_last_matching_category_wins() {
        let ids = strings(&["Basic", "Extensions/Json"]);
        let folders = strings(&["lib/Basic", "lib/Extensions/Json"]);
        let cats = [
            category("Library", &["lib/**"]),
            category("Extensions", &["lib/Extensions/**"]),
        ];
        let l = layout(&cats, &[], &ids, &folders).unwrap();
        assert_eq!(l.of, [(Some(0), String::new()), (Some(1), String::new())]);
        assert_eq!(l.categories[1].tip, "Extensions");
    }

    #[test]
    fn with_groups_every_module_needs_one() {
        let ids = strings(&["A", "B"]);
        let err = layout(&[], &[group("G", &["A", "Z"])], &ids, &ids)
            .err()
            .unwrap();
        assert!(
            err.contains("`Z` (in group `G`) matches no module"),
            "{err}"
        );
        assert!(err.contains("module `B` (B) is in no group"), "{err}");
    }

    #[test]
    fn a_module_folder_pattern_takes_the_modules_inside_it() {
        let ids = strings(&["Android", "Android/Toolchain"]);
        let l = layout(&[], &[group("Platform", &["Android"])], &ids, &ids).unwrap();
        assert_eq!(l.of[1].1, "Platform");
    }

    #[test]
    fn colours_must_be_plain() {
        let mut c = category("X", &["A"]);
        c.color = Some("red;}body{".into());
        let err = layout(&[c], &[], &strings(&["A"]), &strings(&["A"]))
            .err()
            .unwrap();
        assert!(err.contains("not a `#rrggbb` colour"), "{err}");
    }
}
