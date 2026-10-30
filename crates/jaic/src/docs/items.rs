//! Editing the list items that document parameters out of a doc comment.
use super::list_item;

/// `doc` without the list items that document the parameters or results in `names` (see
/// [`list_parameters`]), for a page that shows those apart from the prose.
pub fn without_items(doc: &str, names: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::new();
    let mut fenced = false;
    let mut dropping = false;
    for line in doc.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            dropping = false;
        } else if !fenced {
            if let Some((name, _)) = list_item(trimmed) {
                dropping = names.contains(&name.as_str());
                if dropping {
                    continue;
                }
            } else if dropping && line.starts_with([' ', '\t']) && !trimmed.is_empty() {
                continue;
            } else {
                dropping = false;
            }
        }
        out.push(line);
    }
    // Dropping a list can leave blank lines at its ends or two in a row.
    let mut tidy: Vec<&str> = Vec::new();
    for line in out {
        if line.trim().is_empty() && tidy.last().is_none_or(|l| l.trim().is_empty()) {
            continue;
        }
        tidy.push(line);
    }
    while tidy.last().is_some_and(|l| l.trim().is_empty()) {
        tidy.pop();
    }
    tidy.join("\n")
}
