//! The release-age policy: how old a version must be before `jacket dep` accepts it.
//!
//! `min-release-age` is the floor (default 2 weeks). `release-age-jitter` adds an offset in
//! `[0, jitter]` (default 3 days) that is hashed from the seed, the package and the version, so
//! everyone on one project agrees while different projects wait different times and a
//! compromised release reaches them in stages. A project without a seed hashes the package and
//! version alone, so the offset is still the same on every machine.
use sha2::{Digest, Sha256};

use super::types::Age;

/// "20 hours ago", "3 days ago".
pub fn ago(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let (n, unit) = match seconds {
        s if s < 90 => (s, "second"),
        s if s < 3600 => ((s + 30) / 60, "minute"),
        s if s < 129_600 => ((s + 1800) / 3600, "hour"),
        s => ((s + 43_200) / 86_400, "day"),
    };

    format!(
        "{n} {unit}{} ago",
        if n == 1 {
            ""
        } else {
            "s"
        }
    )
}

/// "3d 5h", "20h", "45m": a duration for messages.
pub fn span(seconds: i64) -> String {
    let (d, h, m) = (
        seconds / 86_400,
        seconds % 86_400 / 3600,
        seconds % 3600 / 60,
    );
    let mut parts = Vec::new();

    for (n, unit) in [(d, 'd'), (h, 'h'), (m, 'm')] {
        if n > 0 {
            parts.push(format!("{n}{unit}"));
        }
    }

    if parts.is_empty() {
        format!("{seconds}s")
    } else {
        parts.join(" ")
    }
}

/// The Unix time of an RFC 3339 timestamp (`2024-05-01T12:00:00Z`, with a fraction or an
/// offset).
pub fn parse_time(text: &str) -> Option<i64> {
    let (date, rest) = text.trim().split_once(['T', 't', ' '])?;
    let mut d = date.split('-');
    let (y, m, day): (i64, i64, i64) = (
        d.next()?.parse().ok()?,
        d.next()?.parse().ok()?,
        d.next()?.parse().ok()?,
    );

    if d.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&day) {
        return None;
    }

    let split = rest.find(['Z', 'z', '+', '-']).unwrap_or(rest.len());
    let (clock, zone) = rest.split_at(split);
    let mut c = clock.split(':');
    let (h, mi): (i64, i64) = (c.next()?.parse().ok()?, c.next()?.parse().ok()?);
    let sec: i64 = c
        .next()
        .map_or(Some(0), |s| s.split('.').next()?.parse().ok())?;
    let offset = match zone.chars().next() {
        None | Some('Z' | 'z') => 0,
        Some(sign) => {
            let (zh, zm) = zone[1..].split_once(':').unwrap_or((&zone[1..], "0"));
            let seconds = zh.parse::<i64>().ok()? * 3600 + zm.parse::<i64>().ok()? * 60;

            if sign == '-' {
                -seconds
            } else {
                seconds
            }
        }
    };

    Some(days_from_civil(y, m, day) * 86_400 + h * 3600 + mi * 60 + sec - offset)
}

/// Days since 1970-01-01 of a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 {
        y - 1
    } else {
        y
    };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy =
        (153 * (if m > 2 {
            m - 3
        } else {
            m + 9
        }) + 2)
            / 5
            + d
            - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    era * 146_097 + doe - 719_468
}

/// The policy that applies to one dependency.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Policy {
    /// How long a version must have been published; `None` means no policy.
    pub min_age: Option<Age>,
    /// The largest jitter.
    pub jitter: Option<Age>,
    /// The project's `[resolver] seed`, if it has one.
    pub seed: Option<String>,
}

impl Policy {
    /// `[resolver]` with the defaults filled in, overridden by the entry's own `min-release-age`
    /// (`"0"` waives everything, the jitter too). A resolver `min-release-age` of `0` also drops
    /// the default jitter; a jitter written beside it is kept.
    pub fn from_config(
        resolver: &jacket_config::Resolver,
        entry: &jacket_config::Dependency,
    ) -> Result<Self, String> {
        let age = |text: &Option<String>| text.as_deref().map(Age::parse).transpose();
        let min_age = match age(&resolver.min_release_age)? {
            Some(age) => age,
            None => Age::parse(jacket_config::Resolver::DEFAULT_MIN_RELEASE_AGE)?,
        };
        let jitter = match age(&resolver.release_age_jitter)? {
            Some(jitter) => Some(jitter),
            None if min_age == Age::ZERO => None,
            None => Some(Age::parse(
                jacket_config::Resolver::DEFAULT_RELEASE_AGE_JITTER,
            )?),
        };
        let mut policy = Self {
            min_age: Some(min_age),
            jitter,
            seed: resolver.seed.clone(),
        };

        if let Some(own) = age(&entry.min_release_age)? {
            policy.min_age = Some(own);

            if own == Age::ZERO {
                policy.jitter = None;
            }
        }

        Ok(policy)
    }

    /// Whether a policy applies at all.
    pub fn active(&self) -> bool {
        self.min_age.is_some_and(|age| age > Age::ZERO)
            || self.jitter.is_some_and(|j| j > Age::ZERO)
    }

    /// The age `tag` of `package` must have reached.
    pub fn required(&self, package: &str, tag: &str) -> i64 {
        let floor = self.min_age.map_or(0, Age::seconds);

        match self.jitter {
            Some(jitter) if jitter > Age::ZERO => {
                let seed = self.seed.as_deref().unwrap_or("");
                floor + offset(seed, package, tag, jitter.seconds())
            }
            _ => floor,
        }
    }
}

/// A repeatable offset in `[0, max]` from the seed, package and version.
pub fn offset(seed: &str, package: &str, version: &str, max: i64) -> i64 {
    let mut hash = Sha256::new();

    for part in [seed, package, version] {
        hash.update(part.as_bytes());
        hash.update([0]);
    }

    let digest = hash.finalize();
    let n = u64::from_be_bytes(digest[..8].try_into().unwrap_or([0; 8]));

    (n % (max as u64 + 1)) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages_read_in_the_largest_fitting_unit() {
        assert_eq!(ago(20 * 3600), "20 hours ago");
        assert_eq!(ago(3600), "1 hour ago");
        assert_eq!(ago(3 * 86_400), "3 days ago");
        assert_eq!(ago(300), "5 minutes ago");
    }

    #[test]
    fn timestamps_parse_with_zones_and_fractions() {
        assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_time("2024-05-01T12:00:00Z"), Some(1_714_564_800));
        assert_eq!(parse_time("2024-05-01T12:00:00.123Z"), Some(1_714_564_800));
        assert_eq!(parse_time("2024-05-01T14:00:00+02:00"), Some(1_714_564_800));
        assert_eq!(parse_time("2024-05-01T07:00:00-05:00"), Some(1_714_564_800));
        assert_eq!(parse_time("2024-02-29T00:00:00Z"), Some(1_709_164_800));
        assert_eq!(parse_time("yesterday"), None);
        assert_eq!(parse_time("2024-13-01T00:00:00Z"), None);
    }

    #[test]
    fn jitter_is_stable_bounded_and_per_project() {
        let a = offset("seed-a", "jaison", "v1.0.0", 172_800);

        assert_eq!(a, offset("seed-a", "jaison", "v1.0.0", 172_800));
        assert!((0..=172_800).contains(&a));
        let differs = (0..8).any(|i| offset(&format!("s{i}"), "jaison", "v1.0.0", 172_800) != a);

        assert!(differs, "another seed gives another offset");
        assert_ne!(offset("seed-a", "jaison", "v1.0.1", 172_800), a);
        assert_eq!(offset("s", "p", "v", 0), 0);
    }

    #[test]
    fn a_project_with_no_settings_waits_two_weeks_plus_up_to_three_days() {
        let resolver = jacket_config::Resolver::default();
        let entry = jacket_config::Dependency::default();
        let policy = Policy::from_config(&resolver, &entry).unwrap();

        assert!(policy.active());
        assert_eq!(policy.min_age, Some(Age::parse("2w").unwrap()));
        assert_eq!(policy.jitter, Some(Age::parse("3d").unwrap()));
        let need = policy.required("jaison", "v1.0.0");

        assert!((1_209_600..=1_209_600 + 259_200).contains(&need));
        // No seed: the offset comes from the package and version, the same on every run.
        assert_eq!(need, policy.required("jaison", "v1.0.0"));
        assert_eq!(
            need,
            1_209_600 + offset("", "jaison", "v1.0.0", 259_200),
            "the offset is hashed from the package and version"
        );
        let seeded = Policy {
            seed: Some("k".into()),
            ..policy.clone()
        };

        assert_ne!(seeded.required("jaison", "v1.0.0"), need);
    }

    #[test]
    fn a_zero_resolver_age_turns_the_policy_off_and_a_written_jitter_is_kept() {
        let entry = jacket_config::Dependency::default();
        let off = jacket_config::Resolver {
            min_release_age: Some("0".into()),
            ..Default::default()
        };
        let policy = Policy::from_config(&off, &entry).unwrap();

        assert!(!policy.active());
        assert_eq!(policy.required("p", "v1"), 0);
        let jittered = jacket_config::Resolver {
            release_age_jitter: Some("1d".into()),
            ..off
        };
        let policy = Policy::from_config(&jittered, &entry).unwrap();

        assert_eq!(policy.jitter, Some(Age::parse("1d").unwrap()));
    }

    #[test]
    fn an_entry_overrides_the_resolver_and_zero_waives_everything() {
        let resolver = jacket_config::Resolver {
            min_release_age: Some("3d".into()),
            release_age_jitter: Some("2d".into()),
            seed: Some("k".into()),
            dedupe: None,
        };
        let plain = jacket_config::Dependency::default();
        let policy = Policy::from_config(&resolver, &plain).unwrap();
        let need = policy.required("p", "v1");

        assert!((259_200..=259_200 + 172_800).contains(&need));
        let waived = jacket_config::Dependency {
            min_release_age: Some("0".into()),
            ..plain.clone()
        };
        let policy = Policy::from_config(&resolver, &waived).unwrap();

        assert!(!policy.active());
        assert_eq!(policy.required("p", "v1"), 0);
        let own = jacket_config::Dependency {
            min_release_age: Some("1h".into()),
            ..plain
        };
        let policy = Policy::from_config(&resolver, &own).unwrap();

        assert!(policy.required("p", "v1") >= 3600);
        assert!(
            Policy::from_config(
                &jacket_config::Resolver {
                    min_release_age: Some("soon".into()),
                    ..resolver
                },
                &own
            )
            .is_err()
        );
    }
}
