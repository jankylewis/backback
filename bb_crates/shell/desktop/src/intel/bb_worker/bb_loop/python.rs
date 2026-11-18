//! pyright's server thread (BBEOD28): its own thread, runtime and process
//! group, so a crash or a missing install never touches Roslyn or
//! rust-analyzer. It serves the window root with the root's interpreter
//! (nearest venv, else `python3` on `PATH`).

use std::path::{Path, PathBuf};
use std::time::Duration;

use bb_intel::PythonConfig;
use bb_python::pyright::Inputs;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{run, Boot};
use crate::intel::bb_worker::protocol::{Event, Request};
use crate::intel::bb_worker::{intel_folder, launch, Link};

/// Start pyright's thread on the folder `root`.
pub(crate) fn start_python(root: &Path) -> (Link, UnboundedReceiver<Event>) {
    let root: PathBuf = root.to_path_buf();
    launch(
        "bb-pyright",
        move |requests: UnboundedReceiver<Request>, events: &UnboundedSender<Event>| {
            serve(&root, requests, events);
        },
    )
}

/// The interpreter pyright resolves imports with for the window `root`.
pub(crate) fn interpreter(root: &Path) -> PathBuf {
    bb_python::interpreter_for(root, root, std::env::var_os("PATH").as_ref()).program
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
    let config: PythonConfig = PythonConfig {
        root: root.to_path_buf(),
        interpreter: interpreter(root),
        inputs: Inputs::from_env(),
        request_timeout: Duration::from_secs(20),
    };
    // `folder` (the log) lives as long as the loop; its drop removes it.
    run(
        Boot::Python(config, folder.path().to_path_buf()),
        requests,
        events,
    );
}
