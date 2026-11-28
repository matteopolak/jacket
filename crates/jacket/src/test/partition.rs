//! `--partition`: which tests a part of a split run keeps.
use super::Case;
use jaic::fxhash::HashMap;
use std::path::Path;

/// `hash:K/N` or `count:K/N`: whether the `index`-th test named `name` is in this part.
pub(super) fn partition(spec: &str) -> Result<impl Fn(usize, &str) -> bool + use<>, String> {
    let bad = || format!("`--partition {spec}` is not `hash:K/N` or `count:K/N` with 1 <= K <= N");
    let (how, part) = spec.split_once(':').ok_or_else(bad)?;
    let (k, n) = part.split_once('/').ok_or_else(bad)?;
    let (k, n): (u64, u64) = (k.parse().map_err(|_| bad())?, n.parse().map_err(|_| bad())?);

    if k == 0 || k > n || !matches!(how, "hash" | "count") {
        return Err(bad());
    }

    let by_hash = how == "hash";

    Ok(move |index: usize, name: &str| {
        let key = if by_hash {
            // FNV-1a: stable across runs and machines.
            name.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
            })
        } else {
            index as u64
        };
        key % n == k - 1
    })
}

/// `time:K/N`: the zero-based part and the part count, or why the spec is wrong.
pub(super) fn time_part(spec: &str) -> Result<(usize, usize), String> {
    let bad = || format!("`--partition {spec}` is not `time:K/N` with 1 <= K <= N");
    let part = spec.strip_prefix("time:").ok_or_else(bad)?;
    let (k, n) = part.split_once('/').ok_or_else(bad)?;
    let (k, n): (usize, usize) = (k.parse().map_err(|_| bad())?, n.parse().map_err(|_| bad())?);

    if k == 0 || k > n {
        return Err(bad());
    }

    Ok((k - 1, n))
}

/// The part (`0..parts`) each of `names` belongs to, so that the parts take about equal
/// recorded time: tests go heaviest first to the part with the least time so far. A test with no
/// entry in `times` weighs the median of the recorded ones (1 s with none). The answer depends
/// only on the set of names and the times, not on their order, so every part computes the same
/// split on its own.
pub(super) fn time_parts(names: &[&str], times: &HashMap<String, f64>, parts: usize) -> Vec<usize> {
    let mut known: Vec<f64> = times.values().copied().collect();

    known.sort_by(f64::total_cmp);
    let median = known.get(known.len() / 2).copied().unwrap_or(1.0);
    let weight = |name: &str| times.get(name).copied().unwrap_or(median);
    let mut order: Vec<usize> = (0..names.len()).collect();

    order.sort_by(|&a, &b| {
        weight(names[b])
            .total_cmp(&weight(names[a]))
            .then_with(|| names[a].cmp(names[b]))
    });
    let mut load = vec![0.0f64; parts];
    let mut assigned = vec![0; names.len()];

    for index in order {
        let lightest = (0..parts)
            .min_by(|&a, &b| load[a].total_cmp(&load[b]))
            .unwrap_or(0);

        load[lightest] += weight(names[index]);
        assigned[index] = lightest;
    }

    assigned
}

/// The recorded seconds per test name, from the JSON object `--partition-times` names.
fn read_times(path: &Path) -> Result<HashMap<String, f64>, String> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "cannot read `--partition-times {}`: {error}",
            path.display()
        )
    })?;

    serde_json::from_str(&text).map_err(|error| {
        format!(
            "`--partition-times {}` is not an object of seconds: {error}",
            path.display()
        )
    })
}

/// This run's share of a `time:K/N` split: its part and the recorded seconds that balance it.
pub(super) struct TimeShare {
    mine: usize,
    parts: usize,
    times: HashMap<String, f64>,
}

impl TimeShare {
    /// The share `--partition SPEC` names (`None` without a spec), with the seconds read from
    /// `--partition-times`.
    pub(super) fn new(spec: Option<&str>, times: Option<&Path>) -> Result<Option<Self>, String> {
        let Some(spec) = spec else {
            return Ok(None);
        };
        let (mine, parts) = time_part(spec)?;
        let times = times.map(read_times).transpose()?.unwrap_or_default();

        Ok(Some(Self {
            mine,
            parts,
            times,
        }))
    }

    /// Drops the cases that belong to the other parts.
    pub(super) fn keep(&self, selected: &mut Vec<Case>) {
        let names: Vec<&str> = selected.iter().map(|case| case.name.as_str()).collect();
        let owner = time_parts(&names, &self.times, self.parts);
        let mut index = 0;

        selected.retain(|_| {
            index += 1;
            owner[index - 1] == self.mine
        });
    }
}

/// A `--partition` split: a per-test filter (`hash`, `count`) or a share of the recorded time.
pub(super) type Filter = Box<dyn Fn(usize, &str) -> bool>;

/// The split `spec` names, with the seconds file `time:` ones read.
pub(super) fn parse(
    spec: Option<&str>,
    times: Option<&Path>,
) -> Result<(Option<Filter>, Option<TimeShare>), String> {
    let by_time = spec.is_some_and(|spec| spec.starts_with("time:"));

    if by_time && times.is_none() {
        return Err(format!(
            "`--partition {}` balances recorded times: pass them with `--partition-times FILE`",
            spec.unwrap_or_default()
        ));
    }

    if !by_time && times.is_some() {
        return Err("`--partition-times` goes only with `time:K/N` partitions".to_string());
    }

    let filter = match spec.filter(|_| !by_time) {
        Some(spec) => Some(Box::new(partition(spec)?) as Filter),
        None => None,
    };
    Ok((filter, TimeShare::new(spec.filter(|_| by_time), times)?))
}
