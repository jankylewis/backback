use std::path::PathBuf;

/// A file argument opens its folder in the Explorer and the file in a tab.
pub(super) fn split_argument(argument: PathBuf) -> (PathBuf, Option<PathBuf>) {
    if argument.is_file() {
        let folder: PathBuf = argument
            .parent()
            .filter(|parent: &&std::path::Path| !parent.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf);
        (folder, Some(argument))
    } else {
        (argument, None)
    }
}
