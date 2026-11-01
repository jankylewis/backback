//! Unit tests for the Roslyn specifics.

use super::*;
use crate::bb_client::Incoming;
use tokio::sync::mpsc;

fn note(method: &str, params: serde_json::Value) -> Incoming {
    Incoming::Notification {
        method: method.to_owned(),
        params: Some(params),
    }
}

#[tokio::test]
async fn wait_for_load_keeps_every_server_message_in_the_log() {
    let (tx, mut rx): (mpsc::Sender<Incoming>, mpsc::Receiver<Incoming>) = mpsc::channel(8);
    tx.send(note(
        "window/logMessage",
        json!({"type": 3, "message": "loading A.csproj"}),
    ))
    .await
    .unwrap();
    tx.send(note(
        "window/showMessage",
        json!({"type": 1, "message": "restore failed"}),
    ))
    .await
    .unwrap();
    tx.send(note("$/progress", json!({"token": 1})))
        .await
        .unwrap();
    tx.send(note(PROJECT_INIT_COMPLETE, json!({})))
        .await
        .unwrap();
    let mut log: Vec<u8> = Vec::new();
    wait_for_load(&mut rx, Duration::from_secs(5), 0, &mut log)
        .await
        .unwrap();
    let text: String = String::from_utf8(log).unwrap();
    assert_eq!(text, "[info] loading A.csproj\n[error] restore failed\n");
}

#[test]
fn settings_turn_automatic_restore_off_by_default_and_on_on_request() {
    assert_eq!(
        settings(Restore::Off).get(AUTOMATIC_RESTORE),
        Some(&json!(false))
    );
    assert_eq!(
        settings(Restore::On).get(AUTOMATIC_RESTORE),
        Some(&json!(true))
    );
}

fn batch(count: usize) -> Incoming {
    note(
        "window/logMessage",
        json!({"type": 3, "message": format!("[LanguageServerProjectSystem] Completed (re)load of {count} project(s) in 00:00:01.5")}),
    )
}

#[tokio::test]
async fn the_init_signal_after_the_first_batch_waits_for_every_project() {
    let (tx, mut rx): (mpsc::Sender<Incoming>, mpsc::Receiver<Incoming>) = mpsc::channel(8);
    tx.send(batch(1)).await.unwrap();
    tx.send(note(PROJECT_INIT_COMPLETE, json!({})))
        .await
        .unwrap();
    tx.send(batch(26)).await.unwrap();
    let mut log: Vec<u8> = Vec::new();
    wait_for_load(&mut rx, Duration::from_secs(5), 27, &mut log)
        .await
        .unwrap();
    let text: String = String::from_utf8(log).unwrap();
    assert!(text.contains("of 26 project(s)"), "{text}");
}

#[tokio::test]
async fn a_load_that_never_reaches_every_project_times_out_with_the_counts() {
    let (tx, mut rx): (mpsc::Sender<Incoming>, mpsc::Receiver<Incoming>) = mpsc::channel(8);
    tx.send(batch(1)).await.unwrap();
    tx.send(note(PROJECT_INIT_COMPLETE, json!({})))
        .await
        .unwrap();
    let mut log: Vec<u8> = Vec::new();
    let waited: Result<Duration, RoslynError> =
        wait_for_load(&mut rx, Duration::from_millis(200), 2, &mut log).await;
    assert!(
        matches!(
            waited,
            Err(RoslynError::LoadTimeout {
                loaded: 1,
                expected: 2,
                ..
            })
        ),
        "{waited:?}"
    );
}
