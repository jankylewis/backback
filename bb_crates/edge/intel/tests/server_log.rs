//! The server's own account: log lines after the load, and typed errors.

#[cfg(test)]
mod common;

use bb_intel::{DocPath, IntelError, TextChange};
use common::{start, start_logged, Harness, DEADLINE};
use shared::SharedLog;

#[cfg(test)]
mod shared {
    use std::sync::{Arc, Mutex};

    /// A log every clone writes to, readable while the session runs.
    #[derive(Clone, Default)]
    pub struct SharedLog(Arc<Mutex<Vec<u8>>>);

    impl SharedLog {
        /// The text written so far.
        pub fn text(&self) -> String {
            String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
        }
    }

    impl std::io::Write for SharedLog {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}

#[tokio::test]
async fn server_log_lines_after_the_load_reach_the_log() {
    let log: SharedLog = SharedLog::default();
    let mut h: Harness = start_logged("afterload", log.clone()).await;
    let at_load: usize = log.text().lines().count();
    let loose: DocPath = h.doc("Core/Loose.cs");
    h.session
        .open_text(&loose, "namespace Harness.Core;\nclass Loose { }\n")
        .await
        .unwrap();
    h.session.diagnostics(&loose, DEADLINE).await.unwrap();
    h.finish().await;
    let text: String = log.text();
    let after: Vec<&str> = text.lines().skip(at_load).collect();
    assert!(
        after
            .iter()
            .any(|l: &&str| l.contains("[textDocument/diagnostic]")),
        "no server line of the pull after the load:\n{text}"
    );
    assert!(after.contains(&"[info] Shutting down"), "{text}");
}

#[tokio::test]
async fn a_formatter_failure_is_a_typed_error_and_the_session_keeps_working() {
    let mut h: Harness = start("formatfail").await;
    let odd: DocPath = h.doc("Core/Odd.cs");
    h.session.open_text(&odd, "[]\u{2029}//").await.unwrap();
    let failed: Result<Vec<TextChange>, IntelError> = h.session.format(&odd, DEADLINE).await;
    let spec: DocPath = h.doc("_NUnitApex/GreeterSpec.cs");
    h.session.open(&spec).await.unwrap();
    let formatted: Result<Vec<TextChange>, IntelError> = h.session.format(&spec, DEADLINE).await;
    h.finish().await;
    assert!(
        matches!(&failed, Err(IntelError::Formatter { code: -32000, message, .. }) if message.contains("CodeShapeAnalyzer")),
        "{failed:?}"
    );
    assert!(formatted.is_ok(), "{formatted:?}");
}
