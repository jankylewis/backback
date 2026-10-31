//! Median and p95 of a small sample set.

use std::time::{Duration, Instant};

use super::RUNS;
use crate::commands::CommandError;

/// Median and p95 (nearest rank) of one item's samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Summary {
    /// Middle sample (the upper one for an even count).
    pub(crate) median: u64,
    /// Nearest-rank 95th percentile; for 5 samples this is the largest.
    pub(crate) p95: u64,
}

impl Summary {
    /// Summarize `samples`; `None` when there are none.
    pub(crate) fn of(samples: &[u64]) -> Option<Self> {
        let mut sorted: Vec<u64> = samples.to_vec();
        sorted.sort_unstable();
        let count: usize = sorted.len();
        let rank: usize = count.checked_mul(95)?.checked_add(99)? / 100;
        let median: u64 = *sorted.get(count / 2)?;
        let p95: u64 = *sorted.get(rank.checked_sub(1)?)?;
        Some(Self { median, p95 })
    }
}

/// One warm-up call, then [`RUNS`] measured calls of `sample`.
pub(crate) fn repeat<F: FnMut() -> Result<u64, CommandError>>(
    mut sample: F,
) -> Result<Summary, CommandError> {
    sample()?;
    let mut samples: Vec<u64> = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        samples.push(sample()?);
    }
    Summary::of(&samples).ok_or(CommandError::Bench(super::BenchError::NoSamples))
}

/// [`repeat`] with the wall time of `op` in microseconds as the sample.
pub(crate) fn timed<F: FnMut() -> Result<(), CommandError>>(
    mut op: F,
) -> Result<Summary, CommandError> {
    repeat(|| -> Result<u64, CommandError> {
        let started: Instant = Instant::now();
        op()?;
        Ok(micros(started.elapsed()))
    })
}

/// Whole microseconds, saturating (a run never lasts 584 000 years).
pub(crate) fn micros(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn five_samples_give_the_middle_and_the_largest() {
        assert_eq!(
            Summary::of(&[50, 10, 40, 20, 30]),
            Some(Summary {
                median: 30,
                p95: 50
            })
        );
        assert_eq!(Summary::of(&[]), None);
        assert_eq!(Summary::of(&[7]), Some(Summary { median: 7, p95: 7 }));
    }

    #[test]
    fn repeat_takes_one_warm_up_and_five_samples() {
        let mut calls: u64 = 0;
        let summary: Summary = repeat(|| -> Result<u64, CommandError> {
            calls += 1;
            Ok(calls)
        })
        .unwrap();
        assert_eq!(calls, 6);
        assert_eq!(summary, Summary { median: 4, p95: 6 });
    }

    proptest! {
        #[test]
        fn median_and_p95_are_samples_in_order(samples in prop::collection::vec(any::<u64>(), 1..64)) {
            let s: Summary = Summary::of(&samples).unwrap();
            prop_assert!(samples.contains(&s.median));
            prop_assert!(samples.contains(&s.p95));
            prop_assert!(s.median <= s.p95);
            let above: usize = samples.iter().filter(|v: &&u64| **v > s.p95).count();
            prop_assert!(above * 20 <= samples.len());
        }
    }
}
