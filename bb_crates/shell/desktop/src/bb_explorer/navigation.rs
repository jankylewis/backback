use std::path::PathBuf;

use bb_explorer::{Entry, Explorer, ExplorerError};

pub(crate) fn right(tree: &mut Explorer, selected: usize) -> Result<usize, ExplorerError> {
    let rows: Vec<(usize, &Entry)> = tree.visible();
    let Some((depth, entry)): Option<&(usize, &Entry)> = rows.get(selected) else {
        return Ok(selected);
    };
    if !entry.expandable() {
        return Ok(selected);
    }
    if !entry.expanded() {
        let path: PathBuf = entry.path().to_path_buf();
        tree.toggle(&path)?;
        return Ok(selected);
    }
    let next: Option<&(usize, &Entry)> = rows.get(selected.saturating_add(1));
    Ok(next
        .filter(|(level, _): &&(usize, &Entry)| level > depth)
        .map_or(selected, |_: &(usize, &Entry)| selected.saturating_add(1)))
}

pub(crate) fn left(tree: &mut Explorer, selected: usize) -> Result<usize, ExplorerError> {
    let rows: Vec<(usize, &Entry)> = tree.visible();
    let Some((depth, entry)): Option<&(usize, &Entry)> = rows.get(selected) else {
        return Ok(selected);
    };
    if entry.expanded() {
        let path: PathBuf = entry.path().to_path_buf();
        tree.toggle(&path)?;
        return Ok(selected);
    }
    if *depth == 0 {
        return Ok(selected);
    }
    Ok(rows
        .iter()
        .take(selected)
        .rposition(|(level, _): &(usize, &Entry)| level < depth)
        .unwrap_or(selected))
}

#[cfg(test)]
mod tests {
    use super::{left, right};
    use bb_explorer::Explorer;
    use std::fs;

    #[test]
    fn right_enters_first_child_of_expanded_folder() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("Core")).unwrap();
        fs::write(dir.path().join("Core/Foo.cs"), "").unwrap();
        let mut tree: Explorer = Explorer::open(dir.path()).unwrap();
        assert_eq!(right(&mut tree, 1).unwrap(), 1);
        assert_eq!(right(&mut tree, 1).unwrap(), 2);
        assert_eq!(left(&mut tree, 2).unwrap(), 1);
    }
}
