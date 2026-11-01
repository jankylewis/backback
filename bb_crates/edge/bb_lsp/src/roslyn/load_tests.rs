//! The load counter on the real Roslyn line sequence (bus 066): readiness
//! comes from distinct project paths, never from summed batch lines.

use super::*;
use crate::bb_client::Incoming;
use bb_dotnet::ProjectPath;
use tokio::sync::mpsc;

const ROOT: &str = "/repo/_Harness";
const NAMES: [&str; 4] = [
    "Core/_Core.csproj",
    "_NUnitApex/_NUnitApex.csproj",
    "XunitPlaywright/_XunitPlaywright.csproj",
    "McpServer/_McpServer.csproj",
];

fn project(name: &str) -> ProjectPath {
    ProjectPath::from_path(std::path::Path::new(ROOT).join(name))
}

fn all() -> Vec<ProjectPath> {
    NAMES.iter().map(|n: &&str| project(n)).collect()
}

fn log(text: &str) -> Incoming {
    Incoming::Notification {
        method: "window/logMessage".to_owned(),
        params: Some(json!({"type": 3, "message": text})),
    }
}

fn done(name: &str) -> Incoming {
    log(&format!(
        "[LanguageServerProjectSystem] Successfully completed load of {ROOT}/{name}"
    ))
}

fn batch(count: usize) -> Incoming {
    log(&format!(
        "[LanguageServerProjectSystem] Completed (re)load of {count} project(s) in 00:00:01.5"
    ))
}

fn signal() -> Incoming {
    Incoming::Notification {
        method: PROJECT_INIT_COMPLETE.to_owned(),
        params: Some(json!({})),
    }
}

async fn run(lines: Vec<Incoming>, deadline: Duration) -> Result<Loaded, RoslynError> {
    let (tx, mut rx): (mpsc::Sender<Incoming>, mpsc::Receiver<Incoming>) = mpsc::channel(32);
    for line in lines {
        tx.send(line).await.unwrap();
    }
    let mut sink: Vec<u8> = Vec::new();
    wait_for_load(&mut rx, deadline, &all(), &mut sink).await
}

#[tokio::test]
async fn the_bus_066_sequence_is_ready_only_after_all_four_distinct_projects() {
    let lines: Vec<Incoming> = vec![
        done(NAMES[0]),
        batch(1),
        signal(),
        done(NAMES[0]),
        done(NAMES[1]),
        done(NAMES[2]),
        batch(3),
        done(NAMES[3]),
        batch(1),
    ];
    let loaded: Loaded = run(lines, Duration::from_secs(5)).await.unwrap();
    assert_eq!(loaded.failed, Vec::<ProjectPath>::new());
}

#[tokio::test]
async fn a_reload_of_a_loaded_project_never_advances_readiness() {
    let lines: Vec<Incoming> = vec![
        done(NAMES[0]),
        batch(1),
        signal(),
        done(NAMES[0]),
        done(NAMES[1]),
        done(NAMES[2]),
        batch(4),
    ];
    let waited: Result<Loaded, RoslynError> = run(lines, Duration::from_millis(200)).await;
    match waited {
        Err(RoslynError::LoadTimeout {
            loaded, missing, ..
        }) => assert_eq!((loaded, missing), (3, vec![project(NAMES[3])])),
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn a_project_that_fails_to_load_is_reported_not_waited_for() {
    let failed: Incoming = log(&format!(
        "[LanguageServerProjectSystem] Error while loading {ROOT}/{}: Warning As Error: NU1605",
        NAMES[1]
    ));
    let lines: Vec<Incoming> = vec![
        done(NAMES[0]),
        signal(),
        failed,
        done(NAMES[2]),
        done(NAMES[3]),
    ];
    let loaded: Loaded = run(lines, Duration::from_secs(5)).await.unwrap();
    assert_eq!(loaded.failed, vec![project(NAMES[1])]);
}

#[tokio::test]
async fn a_failed_project_that_loads_on_retry_is_not_reported() {
    let failed: Incoming = log(&format!(
        "[LanguageServerProjectSystem] Error while loading {ROOT}/{}: restore",
        NAMES[1]
    ));
    let mut lines: Vec<Incoming> = vec![failed, done(NAMES[1]), signal()];
    lines.extend(
        [NAMES[0], NAMES[2], NAMES[3]]
            .iter()
            .map(|n: &&str| done(n)),
    );
    let loaded: Loaded = run(lines, Duration::from_secs(5)).await.unwrap();
    assert_eq!(loaded.failed, Vec::<ProjectPath>::new());
}

#[tokio::test]
async fn the_timeout_error_names_every_missing_project() {
    let waited: Result<Loaded, RoslynError> =
        run(vec![done(NAMES[0]), signal()], Duration::from_millis(200)).await;
    let error: RoslynError = waited.unwrap_err();
    let text: String = error.to_string();
    for name in &NAMES[1..] {
        assert!(text.contains(name), "{text}");
    }
    assert!(text.contains("1 of 4"), "{text}");
}

#[test]
fn every_project_path_spelling_of_one_file_counts_once() {
    let mut state: LoadState = LoadState::new(&all());
    for line in [
        format!("Successfully completed load of {ROOT}/Core/_Core.csproj"),
        format!("Successfully completed load of {ROOT}/Core/./_Core.csproj"),
        format!("Successfully completed load of {ROOT}/McpServer/../Core/_Core.csproj"),
        format!("Successfully completed load of {ROOT}/Other/Outside.csproj"),
    ] {
        state.observe(&line);
    }
    assert_eq!((state.loaded(), state.total()), (1, 4));
}

/// One Roslyn line: project index (0..4 in the solution, 4..6 outside),
/// load or failure, or a batch summary with any count.
fn line_of(event: (usize, u8, usize)) -> String {
    let (index, kind, count): (usize, u8, usize) = event;
    let name: String = NAMES.get(index).map_or_else(
        || format!("Other{index}/O.csproj"),
        |n: &&str| (*n).to_owned(),
    );
    match kind {
        0 => format!("Successfully completed load of {ROOT}/{name}"),
        1 => format!("Error while loading {ROOT}/{name}: reason"),
        _ => format!("Completed (re)load of {count} project(s) in 00:00:01"),
    }
}

proptest::proptest! {
    #[test]
    fn the_state_equals_the_set_model_for_any_line_sequence(
        events in proptest::collection::vec((0usize..6, 0u8..3, 0usize..50), 0..40)
    ) {
        let mut state: LoadState = LoadState::new(&all());
        let mut loaded: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        let mut failed: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for event in &events {
            let changed: bool = state.observe(&line_of(*event));
            let (index, kind, _): (usize, u8, usize) = *event;
            let before: (usize, usize) = (loaded.len(), failed.len());
            if index < NAMES.len() && kind == 0 {
                loaded.insert(index);
                failed.remove(&index);
            } else if index < NAMES.len() && kind == 1 && !loaded.contains(&index) {
                failed.insert(index);
            }
            let moved: bool = before != (loaded.len(), failed.len());
            proptest::prop_assert_eq!(changed, moved, "{:?}", event);
        }
        let names: Vec<ProjectPath> = failed.iter().map(|i: &usize| project(NAMES[*i])).collect();
        let mut got: Vec<ProjectPath> = state.failed();
        got.sort_by(|a: &ProjectPath, b: &ProjectPath| a.as_path().cmp(b.as_path()));
        let mut want: Vec<ProjectPath> = names;
        want.sort_by(|a: &ProjectPath, b: &ProjectPath| a.as_path().cmp(b.as_path()));
        proptest::prop_assert_eq!(state.loaded(), loaded.len());
        proptest::prop_assert_eq!(got, want);
        proptest::prop_assert_eq!(state.settled(), loaded.len() + failed.len() == NAMES.len());
    }
}

#[tokio::test]
async fn a_restore_wait_ends_on_the_restore_end_line_and_logs_what_came_before() {
    let (tx, mut rx): (mpsc::Sender<Incoming>, mpsc::Receiver<Incoming>) = mpsc::channel(8);
    let start: String = format!("Restoring _Core.csproj: Running dotnet restore on {ROOT}/Core");
    for text in [
        start.as_str(),
        "[LanguageServerProjectSystem] Restore complete",
        "after",
    ] {
        tx.send(log(text)).await.unwrap();
    }
    let mut sink: Vec<u8> = Vec::new();
    let ended: bool = wait_for_restore(&mut rx, Duration::from_secs(5), &mut sink)
        .await
        .unwrap();
    let text: String = String::from_utf8(sink).unwrap();
    assert!(ended);
    assert!(
        text.contains("Running dotnet restore") && !text.contains("after"),
        "{text}"
    );
    let silent: bool = wait_for_restore(&mut rx, Duration::from_millis(50), &mut Vec::new())
        .await
        .unwrap();
    assert!(!silent, "a stream with no end line times out as false");
}
