//! Version schemes, tag parsing and ranges live in [`jacket_config::version`]; this adds the
//! tags a remote offers.
pub use jacket_config::version::*;

use super::git::RemoteTag;
use super::types::CommitId;

/// A tag read as a version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tagged {
    pub tag: String,
    pub commit: CommitId,
    pub version: Version,
}

/// The tags that are versions under the scheme, newest first. Ties (two tags naming one
/// version, as `1.0` and `v1.0.0` do) keep the order they came in.
pub fn versions(scheme: Scheme, prefix: Option<&str>, tags: &[RemoteTag]) -> Vec<Tagged> {
    let mut found: Vec<Tagged> = tags
        .iter()
        .filter_map(|t| {
            parse_tag(scheme, prefix, t.name.as_str()).map(|version| Tagged {
                tag: t.name.to_string(),
                commit: t.commit.clone(),
                version,
            })
        })
        .collect();

    found.sort_by(|a, b| b.version.cmp(&a.version));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_newest_first_and_skip_what_is_not_one() {
        let tags: Vec<RemoteTag> = ["v1.2.0", "docs", "v1.10.0", "v1.9.1", "1.10.0-rc.1"]
            .iter()
            .map(|t| RemoteTag {
                name: crate::dep::types::RefName::parse(t).unwrap(),
                commit: CommitId::named(&format!("c-{t}")),
            })
            .collect();
        let names: Vec<String> = versions(Scheme::Semver, None, &tags)
            .into_iter()
            .map(|t| t.tag)
            .collect();

        assert_eq!(names, ["v1.10.0", "1.10.0-rc.1", "v1.9.1", "v1.2.0"]);
    }
}
