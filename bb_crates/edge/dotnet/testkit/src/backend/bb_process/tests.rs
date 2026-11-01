use super::*;

#[tokio::test]
async fn a_timed_out_call_leaves_none_of_its_processes_behind() {
    let pids: PathBuf =
        std::env::temp_dir().join(format!("bb-proc-tree-{}.txt", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_file(&pids);
    let script: String = format!(
        "sleep 30 & echo $! >> {0}; sleep 30 & echo $! >> {0}; wait",
        pids.display()
    );
    let call: Call = Call::new("sh", &std::env::temp_dir()).args(["-c", script.as_str()]);
    let outcome: Result<Finished, TestkitError> = call.finish(Duration::from_millis(500)).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let written: String = std::fs::read_to_string(&pids).unwrap();
    let _gone: std::io::Result<()> = std::fs::remove_file(&pids);
    let children: Vec<&str> = written.lines().collect();
    let alive: Vec<&&str> = children
        .iter()
        .filter(|pid: &&&str| {
            std::process::Command::new("kill")
                .args(["-0", pid])
                .status()
                .unwrap()
                .success()
        })
        .collect();
    assert!(
        matches!(outcome, Err(TestkitError::Timeout { .. })),
        "{outcome:?}"
    );
    assert_eq!(children.len(), 2, "{written}");
    assert!(alive.is_empty(), "children left running: {alive:?}");
}

#[tokio::test]
async fn stop_kills_the_runner_and_its_child() {
    let pids: PathBuf =
        std::env::temp_dir().join(format!("bb-stop-tree-{}.txt", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_file(&pids);
    let script: String = format!("sleep 30 & echo $! >> {}; wait", pids.display());
    let stop: crate::StopFlag = crate::StopFlag::new();
    let call: Call = Call::new("sh", &std::env::temp_dir())
        .args(["-c", script.as_str()])
        .stoppable(stop.clone());
    let task: tokio::task::JoinHandle<Result<Finished, TestkitError>> =
        tokio::spawn(async move { call.finish(Duration::from_secs(10)).await });
    tokio::time::sleep(Duration::from_millis(300)).await;
    stop.stop();
    let outcome: Result<Finished, TestkitError> = task.await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let written: String = std::fs::read_to_string(&pids).unwrap();
    let _gone: std::io::Result<()> = std::fs::remove_file(&pids);
    assert!(matches!(outcome, Err(TestkitError::Cancelled)));
    for pid in written.lines() {
        assert!(!std::process::Command::new("kill")
            .args(["-0", pid])
            .status()
            .unwrap()
            .success());
    }
}
