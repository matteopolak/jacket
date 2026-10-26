//! Release versions: `0.21.0`, `v0.21.0`, `0.22.0-rc.1`. Ordered by semver precedence (build
//! metadata ignored), so `upgrade` can tell newer from older without caring how the tag is spelled.
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Version(semver::Version);

impl Version {
    /// Parses a tag or version; a leading `v` is allowed.
    pub fn parse(text: &str) -> Result<Version, String> {
        semver::Version::parse(text.trim().trim_start_matches('v'))
            .map(Version)
            .map_err(|_| format!("`{text}` is not a version like 0.22.0"))
    }

    /// The tag a release of this version is published under.
    pub fn tag(text: &str) -> Result<String, String> {
        Version::parse(text)?;
        Ok(format!("v{}", text.trim().trim_start_matches('v')))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp_precedence(&other.0)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).unwrap()
    }

    #[test]
    fn a_leading_v_does_not_matter() {
        assert_eq!(v("v0.21.0"), v("0.21.0"));
    }

    #[test]
    fn numbers_compare_numerically_not_as_text() {
        assert!(v("0.9.0") < v("0.10.0"));
        assert!(v("0.21.0") < v("0.21.1"));
        assert!(v("1.0.0") > v("0.99.99"));
    }

    #[test]
    fn a_prerelease_sorts_before_its_release() {
        assert!(v("0.22.0-rc.1") < v("0.22.0"));
        assert!(v("0.22.0-rc.1") < v("0.22.0-rc.2"));
        assert!(v("0.22.0-rc.2") < v("0.22.0-rc.10"));
        assert!(v("0.22.0-1") < v("0.22.0-alpha"));
        assert!(v("0.21.9") < v("0.22.0-rc.1"));
    }

    #[test]
    fn build_metadata_is_ignored() {
        assert_eq!(v("0.22.0+abc"), v("0.22.0"));
    }

    #[test]
    fn junk_is_refused() {
        for bad in ["", "latest", "0.x.1", "0.22.0-", "../etc", "0.22.0-a/b"] {
            assert!(Version::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn tags_get_one_v() {
        assert_eq!(Version::tag("0.22.0").unwrap(), "v0.22.0");
        assert_eq!(Version::tag("v0.22.0").unwrap(), "v0.22.0");
        assert!(Version::tag("nope").is_err());
    }
}
