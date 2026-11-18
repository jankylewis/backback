//! rust-analyzer's server thread (BBEOD27): its own thread, runtime and
//! process group, so a crash or a missing install never touches Roslyn.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_cargo::analyzer::Inputs;
use bb_intel::RustConfig;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{run, Boot};
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::{intel_folder, launch, Link};

/// The Cargo workspace to serve for a window root: the root itself when it
/// holds a `Cargo.toml`.
pub(crate) fn find_cargo(root: &Path) -> Option<PathBuf> {
    root.join("Cargo.toml")
        .is_file()
        .then(|| root.to_path_buf())
}

/// Start rust-analyzer's thread on the workspace `root`.
pub(crate) fn start_rust(root: &Path) -> (Link, UnboundedReceiver<Event>) {
    let root: PathBuf = root.to_path_buf();
    launch(
        "bb-rust-analyzer",
        move |requests: UnboundedReceiver<Request>, events: &UnboundedSender<Event>| {
            serve(&root, requests, events);
        },
    )
}

fn serve(root: &Path, requests: UnboundedReceiver<Request>, events: &UnboundedSender<Event>) {
    let folder: bb_process::OwnedTempDir = match intel_folder() {
        Ok(folder) => folder,
        Err(error) => {
            let _sent: Result<(), tokio::sync::mpsc::error::SendError<Event>> =
                events.send(Event::Failed(format!("temp folder: {error}")));
            return;
        }
    };
    let config: RustConfig = RustConfig {
        root: root.to_path_buf(),
        target_dir: scratch(root, "check"),
        inputs: Inputs::from_env(),
        request_timeout: Duration::from_secs(20),
    };
    // `folder` (the log) lives as long as the loop; its drop removes it.
    run(
        Boot::Rust(config, folder.path().to_path_buf()),
        requests,
        events,
    );
}

/// A build folder for `root`, one per workspace and `use` ("check" for
/// rust-analyzer, "tests" for the Tests panel) under the system temp
/// folder, kept between starts; never the project's own `target/`
/// (headmate, BBEOD27). Two uses never share one, so neither waits on
/// the other's lock.
pub(crate) fn scratch(root: &Path, use_: &str) -> PathBuf {
    let mut hasher: std::collections::hash_map::DefaultHasher =
        std::collections::hash_map::DefaultHasher::new();
    root.hash(&mut hasher);
    std::env::temp_dir()
        .join("backback-rust")
        .join(format!("{:016x}", hasher.finish()))
        .join(use_)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    #[test]
    fn the_check_folder_is_scratch_per_workspace_and_never_inside_it() {
        let root: &Path = Path::new("/work/shop");
        let target: PathBuf = super::scratch(root, "check");
        assert!(!target.starts_with(root), "{}", target.display());
        assert!(target.starts_with(std::env::temp_dir()));
        assert_eq!(target, super::scratch(root, "check"));
        assert_ne!(target, super::scratch(root, "tests"));
        assert_ne!(target, super::scratch(Path::new("/work/other"), "check"));
    }
}
