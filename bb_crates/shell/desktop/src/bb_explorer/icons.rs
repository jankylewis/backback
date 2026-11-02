use bb_explorer::{Entry, EntryKind};
use bb_ui::{bb_theme::Theme, Rgb};
use gpui::{prelude::*, px, svg, Rgba, Svg};

pub(crate) fn chevron(entry: &Entry) -> &'static str {
    if entry.expandable() {
        if entry.expanded() {
            "⌄"
        } else {
            "›"
        }
    } else {
        " "
    }
}

pub(crate) fn icon(entry: &Entry, tint: Rgba) -> Svg {
    let data: &'static [u8] = match entry.kind() {
        EntryKind::Folder if entry.expanded() => {
            include_bytes!("../../bb_assets/bb_folders/open.svg")
        }
        EntryKind::Folder => include_bytes!("../../bb_assets/bb_folders/bb_closed.svg"),
        EntryKind::CSharp => include_bytes!("../../bb_assets/files/bb_csharp.svg"),
        EntryKind::Markdown => include_bytes!("../../bb_assets/files/markdown.svg"),
        EntryKind::Project => include_bytes!("../../bb_assets/files/project.svg"),
        EntryKind::Solution => include_bytes!("../../bb_assets/files/solution.svg"),
        EntryKind::Other => type_icon(file_type(entry.name())),
    };
    svg()
        .data(data)
        .size(px(18.0))
        .flex_shrink_0()
        .text_color(tint)
}

pub(crate) fn tint(entry: &Entry, theme: &Theme) -> Rgb {
    if entry.ignored() {
        return theme.ignored;
    }
    if entry.dimmed() {
        return theme.text_dim;
    }
    match entry.kind() {
        EntryKind::Folder => theme.icon_folder,
        EntryKind::CSharp => theme.icon_csharp,
        EntryKind::Markdown => theme.icon_markdown,
        EntryKind::Project | EntryKind::Solution => theme.icon_project,
        EntryKind::Other => type_tint(file_type(entry.name()), theme),
    }
}

/// The kind of a file that is not C#, Markdown, a project or a solution:
/// it picks the icon and tint (BBEOD16; mapping after VS Code's default
/// icon theme, icons drawn for backback).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileType {
    Json,
    Xml,
    Build,
    Yaml,
    Razor,
    Sql,
    Shell,
    Script,
    Web,
    Image,
    Config,
    Docker,
    Text,
    Generic,
}

/// The type of a file from its name (case ignored): whole names first,
/// then the extension.
pub(crate) fn file_type(name: &str) -> FileType {
    let lower: String = name.to_ascii_lowercase();
    if lower == "dockerfile"
        || lower.ends_with(".dockerfile")
        || lower.starts_with("docker-compose")
    {
        return FileType::Docker;
    }
    if lower.starts_with(".env") || lower.starts_with(".git") || lower == ".editorconfig" {
        return FileType::Config;
    }
    if matches!(lower.as_str(), "license" | "readme" | "authors" | "notice") {
        return FileType::Text;
    }
    let ext: &str = lower
        .rsplit_once('.')
        .map_or("", |(_, ext): (&str, &str)| ext);
    by_extension(ext)
}

fn by_extension(ext: &str) -> FileType {
    match ext {
        "json" | "jsonc" | "json5" => FileType::Json,
        "xml" | "config" | "resx" | "xaml" | "axaml" | "runsettings" | "nuspec" | "plist"
        | "manifest" => FileType::Xml,
        "props" | "targets" | "fsproj" | "vbproj" | "proj" => FileType::Build,
        "yml" | "yaml" => FileType::Yaml,
        "cshtml" | "razor" => FileType::Razor,
        "sql" => FileType::Sql,
        "sh" | "bash" | "zsh" | "ps1" | "psm1" | "cmd" | "bat" => FileType::Shell,
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" => FileType::Script,
        "html" | "htm" | "css" | "scss" | "less" => FileType::Web,
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "icns" | "webp" | "bmp" => FileType::Image,
        "toml" | "ini" | "editorconfig" | "gitignore" | "dockerignore" => FileType::Config,
        "txt" | "log" | "csv" | "tsv" => FileType::Text,
        _ => FileType::Generic,
    }
}

fn type_icon(kind: FileType) -> &'static [u8] {
    match kind {
        FileType::Json => include_bytes!("../../bb_assets/types/data/bb_json.svg"),
        FileType::Xml => include_bytes!("../../bb_assets/types/data/xml.svg"),
        FileType::Build => include_bytes!("../../bb_assets/types/data/build.svg"),
        FileType::Yaml => include_bytes!("../../bb_assets/types/data/yaml.svg"),
        FileType::Razor => include_bytes!("../../bb_assets/types/bb_code/razor.svg"),
        FileType::Sql => include_bytes!("../../bb_assets/types/bb_code/sql.svg"),
        FileType::Shell => include_bytes!("../../bb_assets/types/bb_code/shell.svg"),
        FileType::Script => include_bytes!("../../bb_assets/types/bb_code/bb_script.svg"),
        FileType::Web => include_bytes!("../../bb_assets/types/bb_code/web.svg"),
        FileType::Image => include_bytes!("../../bb_assets/types/misc/image.svg"),
        FileType::Config => include_bytes!("../../bb_assets/types/misc/config.svg"),
        FileType::Docker => include_bytes!("../../bb_assets/types/misc/docker.svg"),
        FileType::Text => include_bytes!("../../bb_assets/types/misc/bb_text.svg"),
        FileType::Generic => include_bytes!("../../bb_assets/files/generic.svg"),
    }
}

fn type_tint(kind: FileType, theme: &Theme) -> Rgb {
    match kind {
        FileType::Json | FileType::Script => theme.string,
        FileType::Xml | FileType::Docker => theme.keyword,
        FileType::Build | FileType::Razor => theme.icon_project,
        FileType::Yaml | FileType::Web => theme.error,
        FileType::Sql | FileType::Image => theme.accent,
        FileType::Shell => theme.ok,
        FileType::Config | FileType::Text | FileType::Generic => theme.icon_file,
    }
}

#[cfg(test)]
mod tests {
    use super::{file_type, FileType};

    #[test]
    fn well_known_files_get_their_own_type() {
        let cases: [(&str, FileType); 32] = [
            ("appsettings.json", FileType::Json),
            ("global.JSON", FileType::Json),
            ("tsconfig.jsonc", FileType::Json),
            ("App.config", FileType::Xml),
            ("Strings.resx", FileType::Xml),
            ("MainWindow.xaml", FileType::Xml),
            ("test.runsettings", FileType::Xml),
            ("Directory.Build.props", FileType::Build),
            ("Common.targets", FileType::Build),
            ("Lib.fsproj", FileType::Build),
            ("ci.yml", FileType::Yaml),
            ("deploy.yaml", FileType::Yaml),
            ("Index.cshtml", FileType::Razor),
            ("Counter.razor", FileType::Razor),
            ("seed.sql", FileType::Sql),
            ("build.sh", FileType::Shell),
            ("run.ps1", FileType::Shell),
            ("make.cmd", FileType::Shell),
            ("site.js", FileType::Script),
            ("app.ts", FileType::Script),
            ("View.tsx", FileType::Script),
            ("index.html", FileType::Web),
            ("site.css", FileType::Web),
            ("logo.png", FileType::Image),
            ("icon.svg", FileType::Image),
            (".gitignore", FileType::Config),
            (".editorconfig", FileType::Config),
            (".env.local", FileType::Config),
            ("Dockerfile", FileType::Docker),
            ("docker-compose.yml", FileType::Docker),
            ("notes.txt", FileType::Text),
            ("LICENSE", FileType::Text),
        ];
        for (name, want) in cases {
            assert_eq!(file_type(name), want, "{name}");
        }
    }

    #[test]
    fn every_type_has_its_own_valid_svg() {
        let all: [FileType; 14] = [
            FileType::Json,
            FileType::Xml,
            FileType::Build,
            FileType::Yaml,
            FileType::Razor,
            FileType::Sql,
            FileType::Shell,
            FileType::Script,
            FileType::Web,
            FileType::Image,
            FileType::Config,
            FileType::Docker,
            FileType::Text,
            FileType::Generic,
        ];
        let mut seen: std::collections::BTreeSet<&[u8]> = std::collections::BTreeSet::new();
        for kind in all {
            let data: &[u8] = super::type_icon(kind);
            let text: &str = std::str::from_utf8(data).unwrap();
            let doc: roxmltree::Document<'_> = roxmltree::Document::parse(text).unwrap();
            assert_eq!(doc.root_element().tag_name().name(), "svg", "{kind:?}");
            assert!(
                text.contains("currentColor"),
                "{kind:?} takes the theme tint"
            );
            assert!(seen.insert(data), "{kind:?} reuses another icon");
        }
    }

    #[test]
    fn unknown_names_stay_generic() {
        for name in ["data.bin", "Makefile.am", "noext", "archive.tar.gz"] {
            assert_eq!(file_type(name), FileType::Generic, "{name}");
        }
    }
}
