//! Measure the local fixture server after one warm-up and five calls.

use std::path::PathBuf;
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
    Ok(())
}
