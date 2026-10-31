//! A fresh copy of the harness and a session on it (restore off).

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_intel::{Config, DocPath, Session};
use bb_lsp::roslyn::Restore;

/// Deadline for one feature request in these tests.
pub const DEADLINE: Duration = Duration::from_secs(30);

/// The harness copy and a loaded session on it.
pub struct Harness {
    /// The canonical copy root.
    pub root: PathBuf,
    /// The session.
    pub session: Session,
}

impl Harness {
    /// A document of the copy.
    pub fn doc(&self, relative: &str) -> DocPath {
        DocPath::new(self.root.join(relative)).unwrap()
    }

    /// Stop the server and remove the copy.
    pub async fn finish(self) {
        self.session.shutdown().await.unwrap();
        let _gone: std::io::Result<()> = std::fs::remove_dir_all(&self.root);
    }
}

/// Copy the harness to a new temp folder named after `name`, load it.
pub async fn start(name: &str) -> Harness {
    start_logged(name, std::io::sink()).await
}

/// [`start`], with the server log going to `log`.
pub async fn start_logged<W: std::io::Write + Send + 'static>(name: &str, log: W) -> Harness {
    let base: PathBuf =
        std::env::temp_dir().join(format!("bb-intel-{name}-{}", std::process::id()));
    let _fresh: std::io::Result<()> = std::fs::remove_dir_all(&base);
    let source: PathBuf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shell/desktop/fixtures/_Harness");
    copy_tree(&source, &base.join("h"));
    let root: PathBuf = std::fs::canonicalize(base.join("h")).unwrap();
    let config: Config = Config {
        solution: root.join("Harness.slnx"),
        restore: Restore::Off,
        log_dir: base.join("logs"),
        scratch: base.join("scratch"),
        request_timeout: DEADLINE,
        load_timeout: Duration::from_secs(120),
    };
    let session: Session = Session::start(&config, log).await.unwrap();
    Harness { root, session }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path: PathBuf = entry.unwrap().path();
        let target: PathBuf = to.join(path.file_name().unwrap());
        let name: String = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() && name != "obj" && name != "bin" {
            copy_tree(&path, &target);
        } else if path.is_file() {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}
