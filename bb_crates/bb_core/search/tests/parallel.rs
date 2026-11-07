//! BB-PRF-006: the parallel search against one file at a time.

#[cfg(test)]
mod helpers {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::AtomicBool;

    use bb_search::{search, Found, Hit, Query};

    fn tree(files: &[(String, String)]) -> tempfile::TempDir {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        for (name, text) in files {
            let path: PathBuf = dir.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        dir
    }

    /// The search reads files across the cores, yet finds exactly what one
    /// file at a time finds: the same hits in the same order, the same
    /// files count and the same cap, over several batches.
    #[test]
    fn the_parallel_search_equals_one_file_at_a_time() {
        let files: Vec<(String, String)> = (0..1_500)
            .map(|n: usize| {
                let text: String = match n % 4 {
                    0 => format!("needle {n}\nnone\nneedle again {n}\n"),
                    1 => "nothing here\n".to_owned(),
                    2 => format!("x\0binary needle {n}"),
                    _ => format!("{}needle\n", "a".repeat(n)),
                };
                (format!("d{:02}/f{n:04}.txt", n % 7), text)
            })
            .chain(
                // A folder sorts before a sibling whose name only starts
                // with it ("a" < "a-b" < "a.txt"), as the walk lists them.
                ["a/z.txt", "a-b/x.txt", "a.txt", "a/b/c/deep.txt", "A.txt"]
                    .map(|name: &str| (name.to_owned(), "needle\n".to_owned())),
            )
            .collect();
        let dir: tempfile::TempDir = tree(&files);
        let query: Query = Query::parse("needle").unwrap();
        for cap in [0, 1, 7, 150, 300, 800, 1_100, 2_000] {
            let stop: AtomicBool = AtomicBool::new(false);
            let parallel: Found = search(dir.path(), &query, cap, &stop);
            assert_eq!(
                parallel,
                one_at_a_time(dir.path(), &query, cap),
                "cap {cap}"
            );
        }
    }

    /// A stop set before the search ends it at once, as before.
    #[test]
    fn a_stopped_search_says_so() {
        let files: Vec<(String, String)> = vec![("a.txt".to_owned(), "needle\n".to_owned())];
        let dir: tempfile::TempDir = tree(&files);
        let stop: AtomicBool = AtomicBool::new(true);
        let found: Found = search(dir.path(), &Query::parse("needle").unwrap(), 10, &stop);
        assert!(found.stopped && found.hits.is_empty());
    }

    /// The search as it was: one file after another, in name order.
    fn one_at_a_time(root: &Path, query: &Query, cap: usize) -> Found {
        let mut found: Found = Found::default();
        for path in bb_search::files(root) {
            let Ok(bytes): Result<Vec<u8>, std::io::Error> = std::fs::read(&path) else {
                continue;
            };
            if bytes.iter().take(8 * 1024).any(|b: &u8| *b == 0) {
                continue;
            }
            let Ok(text): Result<String, std::string::FromUtf8Error> = String::from_utf8(bytes)
            else {
                continue;
            };
            found.files += 1;
            for (index, line) in text.lines().enumerate() {
                let Some(start): Option<usize> = query.find(line) else {
                    continue;
                };
                if found.hits.len() >= cap {
                    found.capped = true;
                    return found;
                }
                let column: usize = line.get(..start).unwrap().encode_utf16().count();
                found.hits.push(Hit {
                    path: path.clone(),
                    line: u32::try_from(index).unwrap(),
                    column: u32::try_from(column).unwrap(),
                    preview: preview_of(line),
                });
            }
        }
        found
    }

    fn preview_of(line: &str) -> String {
        let trimmed: &str = line.trim();
        match trimmed.char_indices().nth(200) {
            Some((end, _)) => format!("{}…", trimmed.get(..end).unwrap()),
            None => trimmed.to_owned(),
        }
    }
}
