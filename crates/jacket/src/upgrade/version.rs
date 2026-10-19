//! Release versions: `0.21.0`, `v0.21.0`, `0.22.0-rc.1`. Ordered like semver (build metadata
//! ignored), so `upgrade` can tell newer from older without caring how the tag is spelled.
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Version {
    core: Vec<u64>,
    pre: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Ident {
    Num(u64),
    Text(String),
}

impl Ord for Ident {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Ident::Num(a), Ident::Num(b)) => a.cmp(b),
            (Ident::Num(_), Ident::Text(_)) => Ordering::Less,
            (Ident::Text(_), Ident::Num(_)) => Ordering::Greater,
            (Ident::Text(a), Ident::Text(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Version {
    /// Parses a tag or version; a leading `v` is allowed.
    pub fn parse(text: &str) -> Result<Version, String> {
        let bad = || format!("`{text}` is not a version like 0.22.0");
        let body = text.trim().trim_start_matches('v');
        let body = body.split('+').next().unwrap_or("");
        let (core, pre) = match body.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (body, None),
        };
        let core = core
            .split('.')
            .map(|part| part.parse::<u64>().map_err(|_| bad()))
            .collect::<Result<Vec<_>, _>>()?;
        let pre = match pre {
            None => Vec::new(),
            Some("") => return Err(bad()),
            Some(pre) => pre
                .split('.')
                .map(|part| match part.parse::<u64>() {
                    Ok(n) => Ok(Ident::Num(n)),
                    Err(_) if !part.is_empty() && part.bytes().all(is_ident_byte) => {
                        Ok(Ident::Text(part.to_owned()))
                    }
                    Err(_) => Err(bad()),
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        Ok(Version {
            core,
            pre,
        })
    }

    /// The tag a release of this version is published under.
    pub fn tag(text: &str) -> Result<String, String> {
        Version::parse(text)?;
        Ok(format!("v{}", text.trim().trim_start_matches('v')))
    }
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-'
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let width = self.core.len().max(other.core.len());
        for i in 0..width {
            let a = self.core.get(i).copied().unwrap_or(0);
            let b = other.core.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => {}
                unequal => return unequal,
            }
        }
        match (self.pre.is_empty(), other.pre.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => self.pre.cmp(&other.pre),
        }
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
    fn a_leading_v_and_padding_do_not_matter() {
        assert_eq!(v("v0.21.0"), v("0.21.0"));
        assert_eq!(v("0.21"), v("0.21.0"));
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
