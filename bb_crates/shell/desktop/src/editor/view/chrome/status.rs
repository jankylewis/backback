use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, px, Context, IntoElement, Window};

use crate::{color, ExplorerView};

/// The commit this app was built from: the installer writes it into the
/// bundle's Info.plist (`BBCommit`); a build outside a bundle says `dev`.
/// Shown so a stale install is visible (BB-FLW-001).
pub(crate) fn build() -> &'static str {
    static BUILD: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BUILD.get_or_init(|| {
        std::env::current_exe()
            .ok()
            .and_then(|exe: std::path::PathBuf| {
                let plist: std::path::PathBuf = exe.parent()?.parent()?.join("Info.plist");
                std::fs::read_to_string(plist).ok()
            })
            .and_then(|text: String| commit_in(&text))
            .unwrap_or_else(|| String::from("dev"))
    })
}

/// The `BBCommit` value of an Info.plist's text.
fn commit_in(plist: &str) -> Option<String> {
    let after: &str = plist.split_once("<key>BBCommit</key>")?.1.trim_start();
    let value: &str = after.strip_prefix("<string>")?.split_once("</string>")?.0;
    (!value.is_empty()).then(|| value.to_owned())
}

/// The foot of the editor: the latest open or save error on the left, the
/// C# server's load state and the build on the right.
pub(crate) fn status_bar(
    status: Option<&str>,
    intel: &str,
    stopped: bool,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .h(px(24.0))
        .px_2()
        .flex()
        .items_center()
        .justify_between()
        .bg(color(theme.overlay))
        .text_color(color(theme.text))
        .child(
            crate::intel::view::one_line(24.0)
                .flex_1()
                .min_w(px(0.0))
                .child(status.unwrap_or_default().to_owned()),
        )
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .pl_2()
                .text_color(color(theme.text_dim))
                .child(if stopped {
                    String::from("C# server stopped — ")
                } else {
                    format!("{intel} · {}", build())
                })
                .when(stopped, |label: gpui::Div| {
                    label.child(restart_button(theme, cx))
                }),
        )
}

fn restart_button(theme: Theme, cx: &mut Context<ExplorerView>) -> impl IntoElement {
    div()
        .id("restart-server")
        .cursor_pointer()
        .text_color(color(theme.icon_csharp))
        .child("Restart")
        .on_click(cx.listener(
            |view: &mut ExplorerView,
             _: &gpui::ClickEvent,
             window: &mut Window,
             cx: &mut Context<ExplorerView>| {
                crate::intel::glue::restart(view, window, cx);
            },
        ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_commit_comes_from_the_bundles_info_plist() {
        let plist: &str = "<dict>\n<key>CFBundleVersion</key><string>0.0.1</string>\n<key>BBCommit</key><string>dd0ccf8</string>\n</dict>";
        assert_eq!(super::commit_in(plist).as_deref(), Some("dd0ccf8"));
        assert_eq!(super::commit_in("<dict></dict>"), None);
        assert_eq!(
            super::commit_in("<key>BBCommit</key><string></string>"),
            None
        );
    }
}
