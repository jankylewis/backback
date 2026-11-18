//! Find in Files bench: `find_bench <root> <query> [runs]` prints each run's
//! time and the median after one warm-up run.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root: PathBuf = PathBuf::from(
        args.first()
            .ok_or("usage: find_bench <root> <query> [runs]")?,
    );
    let query: bb_search::Query = bb_search::Query::parse(args.get(1).ok_or("no query")?)?;
    let runs: usize = args.get(2).map_or(Ok(5), |n: &String| n.parse::<usize>())?;
    let stop: AtomicBool = AtomicBool::new(false);
    let warm: bb_search::Found = bb_search::search(&root, &query, 2_000, &stop);
    let mut times: Vec<Duration> = Vec::new();
    for _ in 0..runs {
        let started: Instant = Instant::now();
        let found: bb_search::Found = bb_search::search(&root, &query, 2_000, &stop);
        times.push(started.elapsed());
        assert_eq!(found, warm, "every run finds the same");
    }
    times.sort();
    let walked: Instant = Instant::now();
    let listed: usize = bb_search::files(&root).count();
    println!(
        "walk_ms={} files_listed={listed}",
        walked.elapsed().as_millis()
    );
    let ms: Vec<u128> = times.iter().map(Duration::as_millis).collect();
    let median: u128 = ms.get(ms.len() / 2).copied().unwrap_or_default();
    println!(
        "hits={} files={} capped={} runs_ms={ms:?} median_ms={median}",
        warm.hits.len(),
        warm.files,
        warm.capped
    );
    Ok(())
}
