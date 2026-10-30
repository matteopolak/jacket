//! Which category each module is listed under, read from `stdlib/categories.toml`.
use std::collections::BTreeMap;
use std::path::Path;

pub struct Categories {
    /// Category names, in the order the file lists them.
    pub order: Vec<String>,
    of: BTreeMap<String, String>,
}

impl Categories {
    /// The category of the module `id`.
    pub fn of(&self, id: &str) -> &str {
        self.of.get(id).map_or("", String::as_str)
    }

    /// Reads `categories.toml` from `stdlib` and checks it against `ids`, every module there is.
    pub fn load(stdlib: &Path, ids: &[String]) -> Result<Categories, String> {
        let path = stdlib.join("categories.toml");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Categories::parse(&text, ids).map_err(|e| format!("{}:\n  {e}", path.display()))
    }

    pub fn parse(text: &str, ids: &[String]) -> Result<Categories, String> {
        let value: toml::Table = text.parse().map_err(|e| format!("{e}"))?;
        let mut order: Vec<String> = Vec::new();
        let mut of: BTreeMap<String, String> = BTreeMap::new();
        let mut problems: Vec<String> = Vec::new();
        let tables = value
            .get("category")
            .and_then(toml::Value::as_array)
            .map_or(&[][..], Vec::as_slice);
        for table in tables {
            let Some(name) = table.get("name").and_then(toml::Value::as_str) else {
                problems.push("a [[category]] has no `name`".to_owned());
                continue;
            };
            if order.iter().any(|n| n == name) {
                problems.push(format!("category `{name}` is declared twice"));
                continue;
            }
            order.push(name.to_owned());
            let modules = table
                .get("modules")
                .and_then(toml::Value::as_array)
                .map_or(&[][..], Vec::as_slice);
            for module in modules.iter().filter_map(toml::Value::as_str) {
                if !ids.iter().any(|id| id == module) {
                    problems.push(format!("`{module}` (in `{name}`) is not a module"));
                } else if let Some(first) = of.insert(module.to_owned(), name.to_owned()) {
                    problems.push(format!("`{module}` is in both `{first}` and `{name}`"));
                }
            }
        }
        for id in ids {
            if !of.contains_key(id) {
                problems.push(format!("module `{id}` has no category"));
            }
        }
        if problems.is_empty() {
            Ok(Categories {
                order,
                of,
            })
        } else {
            Err(problems.join("\n  "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| (*n).to_owned()).collect()
    }

    #[test]
    fn every_module_gets_its_category() {
        let text = "[[category]]\nname = \"A\"\nmodules = [\"X\"]\n[[category]]\nname = \"B\"\nmodules = [\"Y\"]\n";
        let c = Categories::parse(text, &ids(&["X", "Y"])).unwrap();
        assert_eq!(c.order, ["A", "B"]);
        assert_eq!(c.of("Y"), "B");
    }

    #[test]
    fn a_missing_or_unknown_module_is_an_error() {
        let text = "[[category]]\nname = \"A\"\nmodules = [\"X\", \"Z\"]\n";
        let err = Categories::parse(text, &ids(&["X", "Y"])).err().unwrap();
        assert!(err.contains("`Z` (in `A`) is not a module"), "{err}");
        assert!(err.contains("module `Y` has no category"), "{err}");
    }

    #[test]
    fn a_module_cannot_be_listed_twice() {
        let text = "[[category]]\nname = \"A\"\nmodules = [\"X\"]\n[[category]]\nname = \"B\"\nmodules = [\"X\"]\n";
        let err = Categories::parse(text, &ids(&["X"])).err().unwrap();
        assert!(err.contains("both `A` and `B`"), "{err}");
    }
}
