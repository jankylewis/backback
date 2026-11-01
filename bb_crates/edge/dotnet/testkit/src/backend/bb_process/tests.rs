use super::*;

#[tokio::test]
async fn cancelling_a_run_reaps_its_descendant() {
    let file: PathBuf =
        std::env::temp_dir().join(format!("bb-cancel-descendant-{}.txt", std::process::id()));
    let mut call: Call = Call::new("sh", &std::env::temp_dir())
        .args(["-c", "sleep 30 & echo $! > \"$BB_PID_FILE\"; wait"]);
    call.env
        .push(("BB_PID_FILE".into(), file.display().to_string()));
    let task: tokio::task::JoinHandle<Result<Finished, TestkitError>> =
        tokio::spawn(async move { call.finish(Duration::from_secs(30)).await });
    let started: std::time::Instant = std::time::Instant::now();
    while !file.exists() && started.elapsed() < Duration::from_secs(2) {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let pid: String = std::fs::read_to_string(&file).unwrap().trim().to_owned();
    task.abort();
    let _joined: Result<Result<Finished, TestkitError>, _> = task.await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let alive: bool = std::process::Command::new("/bin/kill")
        .args(["-0", &pid])
        .status()
        .unwrap()
        .success();
    if alive {
        let _cleaned: std::io::Result<std::process::ExitStatus> =
            std::process::Command::new("/bin/kill")
                .args(["-KILL", &pid])
                .status();
    }
    let _removed: std::io::Result<()> = std::fs::remove_file(file);
    assert!(!alive, "cancelled runner left descendant {pid}");
}

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
