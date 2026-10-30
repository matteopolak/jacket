//! The recorded coverage a run is compared with, so a drop fails the run. See `mod.rs`.
//!
//! A baseline holds the lines of each module and of all of them: `{"lines": {"hit": 3, "total": 5},
//! "modules": {"Sort": {"hit": 2, "total": 4}}}`. A module or the total has dropped when its
//! share of covered lines is more than the margin (in percentage points) below the baseline's.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::report::{Row, Tally, total};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Baseline {
    pub lines: Tally,
    pub modules: BTreeMap<String, Tally>,
}

/// A module (or `Total`) that covers less than it did.
#[derive(Clone, Debug, PartialEq)]
pub struct Drop {
    pub name: String,
    pub before: f64,
    pub after: f64,
}

impl Baseline {
    /// What `modules` (rows of modules) cover.
    pub fn of(modules: &[Row]) -> Baseline {
        Baseline {
            lines: total(modules).0,
            modules: modules.iter().map(|r| (r.name.clone(), r.lines)).collect(),
        }
    }

    /// The file's text: one module per line, sorted, so a change shows in a diff.
    pub fn render(&self) -> String {
        let tally = |t: Tally| json!({"hit": t.hit, "total": t.total}).to_string();
        let mut text = format!(
            "{{\n  \"lines\": {},\n  \"modules\": {{\n",
            tally(self.lines)
        );
        let last = self.modules.len().saturating_sub(1);
        for (i, (name, t)) in self.modules.iter().enumerate() {
            let comma = if i == last {
                ""
            } else {
                ","
            };
            let _ = writeln!(text, "    {}: {}{comma}", json!(name), tally(*t));
        }
        text.push_str("  }\n}\n");
        text
    }

    pub fn parse(text: &str) -> Result<Baseline, String> {
        let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let tally = |v: &Value| -> Result<Tally, String> {
            let count = |key: &str| {
                v.get(key)
                    .and_then(Value::as_u64)
                    .map(|n| n as usize)
                    .ok_or_else(|| format!("`{key}` is missing or not a count"))
            };
            Ok(Tally {
                hit: count("hit")?,
                total: count("total")?,
            })
        };
        let lines = tally(value.get("lines").ok_or("no `lines`")?)?;
        let modules = value
            .get("modules")
            .and_then(Value::as_object)
            .ok_or("no `modules`")?
            .iter()
            .map(|(name, v)| Ok((name.clone(), tally(v)?)))
            .collect::<Result<_, String>>()?;
        Ok(Baseline {
            lines,
            modules,
        })
    }

    /// The modules of `self` (the baseline) that `now` covers less of by more than `margin`
    /// points, then the total if it does. A module `now` lacks or a baseline lacks is not
    /// compared.
    pub fn drops(&self, now: &Baseline, margin: f64) -> Vec<Drop> {
        let mut found = Vec::new();
        let mut check = |name: &str, before: Tally, after: Tally| {
            if after.percent() < before.percent() - margin {
                found.push(Drop {
                    name: name.to_string(),
                    before: before.percent(),
                    after: after.percent(),
                });
            }
        };
        for (name, before) in &self.modules {
            if let Some(after) = now.modules.get(name) {
                check(name, *before, *after);
            }
        }
        check("Total", self.lines, now.lines);
        found
    }
}

/// A markdown table of the modules, least covered first, with the change from the baseline.
pub fn markdown(
    modules: &[Row],
    baseline: Option<&Baseline>,
    drops: &[Drop],
    margin: f64,
) -> String {
    let (lines, functions) = total(modules);
    let before = |name: &str, now: Tally| -> String {
        let Some(base) = baseline else {
            return String::new();
        };
        let tally = if name == "Total" {
            Some(base.lines)
        } else {
            base.modules.get(name).copied()
        };
        match tally {
            Some(t) => format!("{:+.1}", now.percent() - t.percent()),
            None => "new".to_string(),
        }
    };
    let mut text = String::from("### Test coverage\n\n");
    if drops.is_empty() {
        let _ = writeln!(
            text,
            "{:.1}% of {} lines and {:.1}% of {} procedures ran.\n",
            lines.percent(),
            lines.total,
            functions.percent(),
            functions.total
        );
    } else {
        let _ = writeln!(
            text,
            "**Coverage dropped by more than {margin} points** in {}.\n",
            drops
                .iter()
                .map(|d| format!("`{}` ({:.1}% to {:.1}%)", d.name, d.before, d.after))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    text.push_str(
        "| Module | Lines | Cover | Procedures | Cover | Change |\n|---|--:|--:|--:|--:|--:|\n",
    );
    let mut sorted: Vec<&Row> = modules.iter().collect();
    sorted.sort_by(|a, b| {
        a.lines
            .percent()
            .total_cmp(&b.lines.percent())
            .then_with(|| a.name.cmp(&b.name))
    });
    let mut line = |name: &str, lines: Tally, functions: Tally| {
        let _ = writeln!(
            text,
            "| {name} | {}/{} | {:.1}% | {}/{} | {:.1}% | {} |",
            lines.hit,
            lines.total,
            lines.percent(),
            functions.hit,
            functions.total,
            functions.percent(),
            before(name, lines)
        );
    };
    line("**Total**", lines, functions);
    for row in sorted {
        line(&row.name, row.lines, row.functions);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(name: &str, hit: usize, total: usize) -> Row {
        Row {
            name: name.into(),
            lines: Tally {
                hit,
                total,
            },
            functions: Tally::default(),
        }
    }

    #[test]
    fn a_baseline_is_written_and_read_back() {
        let baseline = Baseline::of(&[module("Sort", 2, 4), module("Hash \"Table\"", 9, 10)]);
        let text = baseline.render();
        assert_eq!(Baseline::parse(&text).unwrap(), baseline);
        assert!(text.contains("\"Sort\": {\"hit\":2,\"total\":4}"), "{text}");
        assert!(Baseline::parse("{").is_err());
        assert!(Baseline::parse("{\"lines\": {\"hit\": 1}}").is_err());
    }

    #[test]
    fn a_module_that_dropped_by_more_than_the_margin_is_reported() {
        let before = Baseline::of(&[
            module("A", 90, 100),
            module("B", 50, 100),
            module("C", 1, 2),
        ]);
        // A loses 0.4 points, B loses 1, C is gone, D is new.
        let after = Baseline::of(&[
            module("A", 897, 1000),
            module("B", 49, 100),
            module("D", 0, 5),
        ]);
        let drops = before.drops(&after, 0.5);
        let names: Vec<_> = drops.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["B"]);
        assert!((drops[0].before - 50.0).abs() < 1e-9 && (drops[0].after - 49.0).abs() < 1e-9);
        // A module that lost 10 points drags the total down by as much here.
        let lower = Baseline::of(&[module("A", 80, 100)]);
        let names: Vec<_> = Baseline::of(&[module("A", 90, 100)])
            .drops(&lower, 0.5)
            .into_iter()
            .map(|d| d.name)
            .collect();
        assert_eq!(names, ["A", "Total"]);
        assert!(before.drops(&before, 0.0).is_empty());
    }

    #[test]
    fn the_summary_lists_modules_least_covered_first_with_their_change() {
        let before = Baseline::of(&[module("A", 50, 100), module("B", 10, 10)]);
        let rows = [module("B", 10, 10), module("A", 40, 100), module("C", 1, 4)];
        let now = Baseline::of(&rows);
        let drops = before.drops(&now, 0.5);
        let text = markdown(&rows, Some(&before), &drops, 0.5);
        assert!(
            text.contains("**Coverage dropped by more than 0.5 points** in `A` (50.0% to 40.0%)"),
            "{text}"
        );
        let order: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("| ") && !l.starts_with("| Module"))
            .map(|l| l.split(" | ").next().unwrap())
            .collect();
        assert_eq!(order, ["| **Total**", "| C", "| A", "| B"]);
        assert!(
            text.contains("| A | 40/100 | 40.0% | 0/0 | 100.0% | -10.0 |"),
            "{text}"
        );
        assert!(
            text.contains("| C | 1/4 | 25.0% | 0/0 | 100.0% | new |"),
            "{text}"
        );
        assert!(markdown(&rows, None, &[], 0.5).contains("ran."));
    }
}
