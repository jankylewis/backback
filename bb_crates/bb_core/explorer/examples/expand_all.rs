//! Read-only Explorer expansion benchmark.

use std::{path::PathBuf, time::Instant};

use bb_explorer::Explorer;

fn main() {
    let Some(path): Option<PathBuf> = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: expand_all <folder-or-slnx>");
        std::process::exit(2);
    };
    let mut tree: Explorer = match Explorer::open(&path) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let start: Instant = Instant::now();
    match tree.expand_all() {
        Ok(folders) => println!(
            "folders={folders} elapsed_ms={}",
            start.elapsed().as_millis()
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
