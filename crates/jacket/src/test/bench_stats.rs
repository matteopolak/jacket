//! The statistics of `jacket bench`: a least-squares fit through the samples, bootstrap
//! confidence intervals, Tukey outliers and the comparison of two runs. See
//! docs/tools/jacket-bench.md.
//!
//! A sample is a pair (iterations, nanoseconds). In linear sampling the iterations grow as
//! k * step, so the slope of the line through the samples is the time of one iteration. The line
//! goes through the origin, as criterion's does: what every sample costs on top (two clock
//! reads and a call, about 0.1 us) is under 0.02% of the shortest sample, which linear sampling
//! makes last about a millisecond, while a free intercept would multiply the noise of the slope.
//! The intercept of a free fit is still reported, as a check. A run with equal iterations in
//! every sample has no line; its estimate is the mean.

/// The share of the bootstrap distribution inside an interval.
pub const CONFIDENCE: f64 = 0.95;

/// A small, fast, seeded generator (splitmix64): the same samples always give the same report.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`, without modulo bias.
    fn below(&mut self, n: usize) -> usize {
        ((u128::from(self.next()) * n as u128) >> 64) as usize
    }
}

/// An estimate and the interval around it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub low: f64,
    pub estimate: f64,
    pub high: f64,
}

/// The raw measurements of one benchmark.
#[derive(Clone, Debug, PartialEq)]
pub struct Samples {
    pub iterations: Vec<f64>,
    pub nanoseconds: Vec<f64>,
}

impl Samples {
    pub fn len(&self) -> usize {
        self.iterations.len()
    }

    /// Nanoseconds per iteration of every sample.
    pub fn per_iteration(&self) -> Vec<f64> {
        self.iterations
            .iter()
            .zip(&self.nanoseconds)
            .map(|(i, ns)| ns / i)
            .collect()
    }

    /// Whether the samples have different iteration counts, so a line can be fitted.
    pub fn is_linear(&self) -> bool {
        self.iterations
            .iter()
            .any(|&i| (i - self.iterations[0]).abs() > 0.5)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Estimator {
    /// The slope of the least-squares line through (iterations, nanoseconds).
    Slope,
    /// Total nanoseconds over total iterations.
    Mean,
}

impl Estimator {
    pub fn name(self) -> &'static str {
        match self {
            Estimator::Slope => "slope",
            Estimator::Mean => "mean",
        }
    }
}

/// A line fitted through the samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub slope: f64,
    pub intercept: f64,
    pub r_squared: f64,
}

/// Least squares of `y = slope * x`, with the R-squared against the mean of `y`.
pub fn fit_through_origin(x: &[f64], y: &[f64]) -> Option<Fit> {
    let sxx: f64 = x.iter().map(|a| a * a).sum();
    if x.len() < 2 || sxx <= 0.0 {
        return None;
    }
    let slope = x.iter().zip(y).map(|(a, b)| a * b).sum::<f64>() / sxx;
    let my = y.iter().sum::<f64>() / y.len() as f64;
    let residual: f64 = x.iter().zip(y).map(|(a, b)| (b - slope * a).powi(2)).sum();
    let total: f64 = y.iter().map(|b| (b - my).powi(2)).sum();
    Some(Fit {
        slope,
        intercept: 0.0,
        r_squared: if total <= 0.0 {
            1.0
        } else {
            (1.0 - residual / total).clamp(0.0, 1.0)
        },
    })
}

/// Ordinary least squares of `y` on `x`. `None` when all `x` are equal.
pub fn fit(x: &[f64], y: &[f64]) -> Option<Fit> {
    let n = x.len() as f64;
    if x.len() < 2 {
        return None;
    }
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for (&a, &b) in x.iter().zip(y) {
        sxx += (a - mx) * (a - mx);
        sxy += (a - mx) * (b - my);
        syy += (b - my) * (b - my);
    }
    if sxx <= 0.0 {
        return None;
    }
    let slope = sxy / sxx;
    let r_squared = if syy <= 0.0 {
        1.0
    } else {
        (sxy * sxy / (sxx * syy)).clamp(0.0, 1.0)
    };
    Some(Fit {
        slope,
        intercept: my - slope * mx,
        r_squared,
    })
}

pub fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

/// The sample variance (n - 1).
fn variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    values.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (values.len() - 1) as f64
}

/// The median of values that are already sorted.
pub fn median_sorted(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n == 0 {
        0.0
    } else if n % 2 == 1 {
        sorted[n / 2]
    } else {
        f64::midpoint(sorted[n / 2 - 1], sorted[n / 2])
    }
}

pub fn median(values: &[f64]) -> f64 {
    median_sorted(&sorted(values))
}

pub fn sorted(values: &[f64]) -> Vec<f64> {
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    v
}

/// The value `percent` percent of the way through sorted values, between neighbours.
pub fn percentile(sorted: &[f64], percent: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let position = (sorted.len() - 1) as f64 * percent / 100.0;
    let low = position.floor() as usize;
    let high = (low + 1).min(sorted.len() - 1);
    sorted[low] + (sorted[high] - sorted[low]) * (position - low as f64)
}

/// The median absolute deviation.
pub fn mad(values: &[f64]) -> f64 {
    let m = median(values);
    median(&values.iter().map(|v| (v - m).abs()).collect::<Vec<_>>())
}

/// The interval holding `CONFIDENCE` of a bootstrap distribution.
fn interval(estimate: f64, mut distribution: Vec<f64>) -> Interval {
    distribution.retain(|v| v.is_finite());
    if distribution.is_empty() {
        return Interval {
            low: estimate,
            estimate,
            high: estimate,
        };
    }
    distribution.sort_by(f64::total_cmp);
    let tail = (1.0 - CONFIDENCE) / 2.0 * 100.0;
    Interval {
        low: percentile(&distribution, tail).min(estimate),
        estimate,
        high: percentile(&distribution, 100.0 - tail).max(estimate),
    }
}

/// The estimator over the samples picked by `pick`.
fn estimate(samples: &Samples, estimator: Estimator, pick: &[usize]) -> f64 {
    match estimator {
        Estimator::Slope => {
            let x: Vec<f64> = pick.iter().map(|&i| samples.iterations[i]).collect();
            let y: Vec<f64> = pick.iter().map(|&i| samples.nanoseconds[i]).collect();
            fit_through_origin(&x, &y).map_or(f64::NAN, |f| f.slope)
        }
        Estimator::Mean => {
            let ns: f64 = pick.iter().map(|&i| samples.nanoseconds[i]).sum();
            let it: f64 = pick.iter().map(|&i| samples.iterations[i]).sum();
            ns / it
        }
    }
}

fn estimator_of(samples: &Samples) -> Estimator {
    if samples.is_linear() && samples.len() >= 3 {
        Estimator::Slope
    } else {
        Estimator::Mean
    }
}

/// Bootstraps `resample` times: each call fills `pick` with random indices into `0..n`.
fn bootstrap(
    n: usize,
    resamples: usize,
    rng: &mut Rng,
    mut statistic: impl FnMut(&[usize]) -> f64,
) -> Vec<f64> {
    let mut pick = vec![0; n];
    (0..resamples)
        .map(|_| {
            for slot in &mut pick {
                *slot = rng.below(n);
            }
            statistic(&pick)
        })
        .collect()
}

/// How many measurements fall outside Tukey's fences (1.5 and 3 times the interquartile range).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outliers {
    pub low_severe: usize,
    pub low_mild: usize,
    pub high_mild: usize,
    pub high_severe: usize,
}

impl Outliers {
    pub fn classify(per_iteration: &[f64]) -> Outliers {
        let sorted = sorted(per_iteration);
        let (q1, q3) = (percentile(&sorted, 25.0), percentile(&sorted, 75.0));
        let iqr = q3 - q1;
        let mut found = Outliers::default();
        for &v in &sorted {
            if v < q1 - 3.0 * iqr {
                found.low_severe += 1;
            } else if v < q1 - 1.5 * iqr {
                found.low_mild += 1;
            } else if v > q3 + 3.0 * iqr {
                found.high_severe += 1;
            } else if v > q3 + 1.5 * iqr {
                found.high_mild += 1;
            }
        }
        found
    }

    pub fn total(&self) -> usize {
        self.low_severe + self.low_mild + self.high_mild + self.high_severe
    }
}

/// What a run says about one benchmark.
#[derive(Clone, Debug, PartialEq)]
pub struct Analysis {
    pub estimator: Estimator,
    /// The time of one iteration by `estimator`, with its bootstrap interval.
    pub time: Interval,
    pub mean: Interval,
    pub median: Interval,
    pub mad: f64,
    /// How well the line through the origin fits, and the intercept of a free fit (what each
    /// sample seems to cost besides its iterations; noisy, a check only). `None` when every
    /// sample has the same number of iterations.
    pub r_squared: Option<f64>,
    pub overhead: Option<f64>,
    pub outliers: Outliers,
}

pub fn analyze(samples: &Samples, resamples: usize, rng: &mut Rng) -> Analysis {
    let n = samples.len();
    let per_iteration = samples.per_iteration();
    let estimator = estimator_of(samples);
    let all: Vec<usize> = (0..n).collect();

    let time_estimate = estimate(samples, estimator, &all);
    let time = interval(
        time_estimate,
        bootstrap(n, resamples, rng, |pick| estimate(samples, estimator, pick)),
    );
    let mean_estimate = mean(&per_iteration);
    let mean = interval(
        mean_estimate,
        bootstrap(n, resamples, rng, |pick| {
            mean(&pick.iter().map(|&i| per_iteration[i]).collect::<Vec<_>>())
        }),
    );
    let median_estimate = median(&per_iteration);
    let mut scratch = Vec::with_capacity(n);
    let median = interval(
        median_estimate,
        bootstrap(n, resamples, rng, |pick| {
            scratch.clear();
            scratch.extend(pick.iter().map(|&i| per_iteration[i]));
            scratch.sort_by(f64::total_cmp);
            median_sorted(&scratch)
        }),
    );
    let line = (estimator == Estimator::Slope)
        .then(|| fit_through_origin(&samples.iterations, &samples.nanoseconds))
        .flatten();
    let free = (estimator == Estimator::Slope)
        .then(|| fit(&samples.iterations, &samples.nanoseconds))
        .flatten();
    Analysis {
        estimator,
        time,
        mean,
        median,
        mad: mad(&per_iteration),
        r_squared: line.map(|f| f.r_squared),
        overhead: free.map(|f| f.intercept),
        outliers: Outliers::classify(&per_iteration),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Improved,
    Regressed,
    WithinNoise,
    NoChange,
}

impl Verdict {
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Improved => "improved",
            Verdict::Regressed => "regressed",
            Verdict::WithinNoise => "within_noise",
            Verdict::NoChange => "no_change",
        }
    }

    pub fn sentence(self) -> &'static str {
        match self {
            Verdict::Improved => "Performance has improved.",
            Verdict::Regressed => "Performance has regressed.",
            Verdict::WithinNoise => "Change within noise threshold.",
            Verdict::NoChange => "No change in performance detected.",
        }
    }
}

/// How a run compares with an earlier one.
#[derive(Clone, Debug, PartialEq)]
pub struct Comparison {
    /// The relative change of the time per iteration (0.05 is 5% slower) and its interval.
    pub change: Interval,
    pub p_value: f64,
    pub verdict: Verdict,
}

/// Welch's t statistic of two sets of values.
fn t_statistic(a: &[f64], b: &[f64]) -> f64 {
    let difference = mean(a) - mean(b);
    let spread = (variance(a) / a.len() as f64 + variance(b) / b.len() as f64).sqrt();
    if spread > 0.0 {
        difference / spread
    } else if difference == 0.0 {
        0.0
    } else {
        f64::INFINITY.copysign(difference)
    }
}

/// Compares `now` with `before`.
///
/// The change is the ratio of the two time estimates minus one, with a bootstrap interval from
/// resampling each run on its own. The p-value is the share of Welch t statistics, drawn from
/// both runs pooled (so there is no difference), at least as extreme as the one observed. A
/// change counts when p is below `significance` and the whole interval lies beyond
/// `noise_threshold` (a fraction) on one side.
pub fn compare(
    now: &Samples,
    before: &Samples,
    resamples: usize,
    noise_threshold: f64,
    significance: f64,
    rng: &mut Rng,
) -> Option<Comparison> {
    let estimator =
        if estimator_of(now) == Estimator::Slope && estimator_of(before) == Estimator::Slope {
            Estimator::Slope
        } else {
            Estimator::Mean
        };
    let all = |s: &Samples| (0..s.len()).collect::<Vec<_>>();
    let (new_time, old_time) = (
        estimate(now, estimator, &all(now)),
        estimate(before, estimator, &all(before)),
    );
    if !(old_time > 0.0 && new_time.is_finite()) {
        return None;
    }
    let point = new_time / old_time - 1.0;

    let (mut new_pick, mut old_pick) = (vec![0; now.len()], vec![0; before.len()]);
    let changes: Vec<f64> = (0..resamples)
        .map(|_| {
            for slot in &mut new_pick {
                *slot = rng.below(now.len());
            }
            for slot in &mut old_pick {
                *slot = rng.below(before.len());
            }
            estimate(now, estimator, &new_pick) / estimate(before, estimator, &old_pick) - 1.0
        })
        .collect();
    let change = interval(point, changes);

    let (a, b) = (now.per_iteration(), before.per_iteration());
    let observed = t_statistic(&a, &b).abs();
    let pool: Vec<f64> = a.iter().chain(&b).copied().collect();
    let (mut left, mut right) = (vec![0.0; a.len()], vec![0.0; b.len()]);
    let mut extreme = 0usize;
    for _ in 0..resamples {
        for slot in &mut left {
            *slot = pool[rng.below(pool.len())];
        }
        for slot in &mut right {
            *slot = pool[rng.below(pool.len())];
        }
        if t_statistic(&left, &right).abs() >= observed {
            extreme += 1;
        }
    }
    let p_value = extreme as f64 / resamples.max(1) as f64;

    let verdict = if p_value >= significance {
        Verdict::NoChange
    } else if change.low > noise_threshold {
        Verdict::Regressed
    } else if change.high < -noise_threshold {
        Verdict::Improved
    } else {
        Verdict::WithinNoise
    };
    Some(Comparison {
        change,
        p_value,
        verdict,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Samples of a benchmark that takes `per` ns an iteration, with `overhead` ns per sample
    /// and a deterministic wobble of `noise` (a fraction).
    fn linear(per: f64, overhead: f64, noise: f64, count: usize, seed: u64) -> Samples {
        let mut rng = Rng::new(seed);
        let mut samples = Samples {
            iterations: vec![],
            nanoseconds: vec![],
        };
        for k in 1..=count {
            let iterations = (k * 1000) as f64;
            let wobble = 1.0 + noise * (rng.below(2001) as f64 / 1000.0 - 1.0);
            samples.iterations.push(iterations);
            samples
                .nanoseconds
                .push((overhead + per * iterations) * wobble);
        }
        samples
    }

    #[test]
    fn the_fit_recovers_slope_and_overhead() {
        let x: Vec<f64> = (1..=20).map(f64::from).collect();
        let y: Vec<f64> = x.iter().map(|v| 3.0 * v + 40.0).collect();
        let f = fit(&x, &y).unwrap();
        assert!((f.slope - 3.0).abs() < 1e-9 && (f.intercept - 40.0).abs() < 1e-9);
        assert!((f.r_squared - 1.0).abs() < 1e-12);
        assert!(fit(&[2.0, 2.0, 2.0], &[1.0, 2.0, 3.0]).is_none());
    }

    #[test]
    fn a_small_fixed_cost_per_sample_hardly_moves_the_estimate() {
        // The shortest sample lasts 2 us here, and the clock reads and the call cost 100 ns.
        let samples = linear(2.0, 100.0, 0.0, 30, 1);
        let analysis = analyze(&samples, 2000, &mut Rng::new(7));
        assert_eq!(analysis.estimator, Estimator::Slope);
        assert!((analysis.time.estimate - 2.0).abs() < 0.01, "{analysis:?}");
        assert!((analysis.overhead.unwrap() - 100.0).abs() < 1e-6);
        assert!(analysis.r_squared.unwrap() > 0.999);
        let through = fit_through_origin(&samples.iterations, &samples.nanoseconds).unwrap();
        assert_eq!(through.intercept, 0.0);
    }

    #[test]
    fn the_interval_holds_the_estimate_and_is_narrow_for_quiet_samples() {
        let samples = linear(10.0, 300.0, 0.01, 50, 3);
        let analysis = analyze(&samples, 5000, &mut Rng::new(9));
        let t = analysis.time;
        assert!(t.low <= t.estimate && t.estimate <= t.high);
        assert!(t.high - t.low < 0.05 * t.estimate, "{t:?}");
        assert!(analysis.median.low <= analysis.median.high);
    }

    #[test]
    fn equal_iterations_fall_back_to_the_mean() {
        let samples = Samples {
            iterations: vec![100.0; 12],
            nanoseconds: (0..12).map(|i| 1000.0 + f64::from(i)).collect(),
        };
        let analysis = analyze(&samples, 1000, &mut Rng::new(1));
        assert_eq!(analysis.estimator, Estimator::Mean);
        assert!(analysis.r_squared.is_none());
        assert!((analysis.time.estimate - 10.055).abs() < 1e-9);
    }

    #[test]
    fn the_same_data_gives_the_same_report() {
        let samples = linear(4.0, 100.0, 0.05, 40, 5);
        let first = analyze(&samples, 1000, &mut Rng::new(11));
        let second = analyze(&samples, 1000, &mut Rng::new(11));
        assert_eq!(first, second);
    }

    #[test]
    fn tukey_fences_split_mild_from_severe() {
        let mut values: Vec<f64> = (0..40).map(|i| 100.0 + f64::from(i % 5)).collect();
        values.extend([107.0, 160.0, 40.0]);
        let found = Outliers::classify(&values);
        assert_eq!(found.high_severe, 1, "{found:?}");
        assert_eq!(found.low_severe, 1, "{found:?}");
        assert_eq!(found.total(), found.low_mild + found.high_mild + 2);
        assert_eq!(Outliers::classify(&[5.0; 10]).total(), 0);
    }

    #[test]
    fn identical_runs_show_no_change() {
        let a = linear(10.0, 200.0, 0.02, 40, 1);
        let b = linear(10.0, 200.0, 0.02, 40, 2);
        let c = compare(&a, &b, 5000, 0.02, 0.05, &mut Rng::new(3)).unwrap();
        assert_ne!(c.verdict, Verdict::Regressed);
        assert_ne!(c.verdict, Verdict::Improved);
        assert!(c.change.low <= c.change.estimate && c.change.estimate <= c.change.high);
    }

    #[test]
    fn a_clear_slowdown_regresses_and_a_speedup_improves() {
        let base = linear(10.0, 200.0, 0.01, 40, 1);
        let slower = linear(12.0, 200.0, 0.01, 40, 2);
        let c = compare(&slower, &base, 5000, 0.02, 0.05, &mut Rng::new(3)).unwrap();
        assert_eq!(c.verdict, Verdict::Regressed);
        assert!((c.change.estimate - 0.2).abs() < 0.02, "{c:?}");
        assert!(c.p_value < 0.05);
        let c = compare(&base, &slower, 5000, 0.02, 0.05, &mut Rng::new(3)).unwrap();
        assert_eq!(c.verdict, Verdict::Improved);
    }

    #[test]
    fn a_significant_change_under_the_threshold_is_within_noise() {
        let base = linear(10.0, 200.0, 0.002, 40, 1);
        let barely = linear(10.05, 200.0, 0.002, 40, 2);
        let c = compare(&barely, &base, 5000, 0.02, 0.05, &mut Rng::new(3)).unwrap();
        assert!(
            matches!(c.verdict, Verdict::WithinNoise | Verdict::NoChange),
            "{c:?}"
        );
    }
}
