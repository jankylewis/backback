//! Measure the local fixture server: cold start, five warm calls, five stops.

use std::path::{Path, PathBuf};
use std::time::Instant;

use bb_mcp::Session;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../shell/desktop/fixtures/_Harness/McpServer/_McpServer.csproj");
    let started: Instant = Instant::now();
    let mut session: Session = Session::start(&project).await?;
    let tools: Vec<bb_mcp::Tool> = session.tools().await?;
    let cold_ms: u128 = started.elapsed().as_millis();
    session.call("Greet", json!({"name":"Vinh"})).await?;
    let mut samples: Vec<u128> = Vec::new();
    for _run in 0..5 {
        let before: Instant = Instant::now();
        session.call("Greet", json!({"name":"Vinh"})).await?;
        samples.push(before.elapsed().as_micros());
    }
    samples.sort_unstable();
    println!(
        "cold_start_to_tools_list_ms={cold_ms} tools={}",
        tools.len()
    );
    println!(
        "warm_call_p50_us={}",
        samples.get(2).copied().unwrap_or_default()
    );
    session.stop().await?;
    let stops: Vec<u128> = stop_samples(&project).await?;
    println!(
        "stop_ms={stops:?} stop_median_ms={}",
        stops.get(2).copied().unwrap_or_default()
    );
    Ok(())
}

/// One warm-up stop, then five timed graceful stops, sorted.
async fn stop_samples(project: &Path) -> Result<Vec<u128>, Box<dyn std::error::Error>> {
    let mut samples: Vec<u128> = Vec::new();
    for run in 0..6 {
        let session: Session = Session::start(project).await?;
        let before: Instant = Instant::now();
        let status: std::process::ExitStatus = session.stop().await?;
        if !status.success() {
            return Err(format!("stop ended with {status}").into());
        }
        if run > 0 {
            samples.push(before.elapsed().as_millis());
        }
    }
    samples.sort_unstable();
    Ok(samples)
}
