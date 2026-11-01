use super::super::model::{enum_label, FieldKind, FormField, Server, ToolView};
use super::button;
use crate::{color, ExplorerView};
use bb_ui::bb_theme::Theme;
use gpui::{div, prelude::*, AnyElement, Context, IntoElement, StatefulInteractiveElement, Window};

pub(super) fn tools(
    view: &ExplorerView,
    server: &Server,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> AnyElement {
    let list: Vec<AnyElement> = tool_list(view.harness.tool, server, theme, cx);
    let mut body: gpui::Div = div().flex_1().children(list);
    if let Some(tool) = view
        .harness
        .tool
        .and_then(|index: usize| server.tools.get(index))
    {
        body = body.child(tool_form(view, tool, theme, cx));
    }
    if let Some(result) = &server.result {
        body = body.child(super::result::result_view(view, result.rows(), theme, cx));
    }
    div()
        .id("tools-scroll")
        .flex_1()
        .overflow_y_scroll()
        .child(body)
        .into_any_element()
}

fn tool_list(
    active: Option<usize>,
    server: &Server,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> Vec<AnyElement> {
    server
        .tools
        .iter()
        .enumerate()
        .map(|(index, tool): (usize, &ToolView)| {
            div()
                .id(format!("tool-{index}"))
                .px_2()
                .py_1()
                .bg(color(if active == Some(index) {
                    theme.overlay
                } else {
                    theme.surface
                }))
                .child(tool.name.clone())
                .on_click(cx.listener(
                    move |view: &mut ExplorerView,
                          _: &gpui::ClickEvent,
                          _: &mut Window,
                          cx: &mut Context<ExplorerView>| {
                        view.harness.tool = Some(index);
                        view.harness.edit = None;
                        cx.notify();
                    },
                ))
                .into_any_element()
        })
        .collect()
}

fn tool_form(
    view: &ExplorerView,
    tool: &ToolView,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    div()
        .child(div().px_2().py_1().child(tool.description.clone()))
        .children(
            tool.fields
                .iter()
                .enumerate()
                .map(|(index, field): (usize, &FormField)| {
                    field_row(index, field, view, theme, cx).into_any_element()
                })
                .collect::<Vec<AnyElement>>(),
        )
        .child(button(
            "call-tool".into(),
            "Call",
            theme,
            cx,
            |view: &mut ExplorerView, cx: &mut Context<ExplorerView>| {
                view.harness.call();
                cx.notify();
            },
        ))
}

fn field_row(
    index: usize,
    field: &FormField,
    view: &ExplorerView,
    theme: Theme,
    cx: &mut Context<ExplorerView>,
) -> impl IntoElement {
    let label: String = field.path.join(".");
    let value: String = if field.value.is_empty() {
        "<empty>".into()
    } else {
        field.value.clone()
    };
    let suffix: &'static str = match field.kind {
        FieldKind::Boolean => " (click to toggle)",
        FieldKind::Enum(_) => " (click to choose)",
        FieldKind::Integer => " (integer)",
        FieldKind::Number => " (number)",
        FieldKind::Array(_) => " (a, b or JSON array)",
        FieldKind::Json => " (JSON)",
        FieldKind::String => "",
    };
    div()
        .id(format!("field-{index}"))
        .px_2()
        .py_1()
        .flex()
        .flex_col()
        .bg(color(if view.harness.edit == Some(index) {
            theme.overlay
        } else {
            theme.surface
        }))
        .child(format!("{label}{suffix}"))
        .child(div().text_color(color(theme.text_dim)).child(value))
        .on_click(cx.listener(
            move |view: &mut ExplorerView,
                  _: &gpui::ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<ExplorerView>| {
                activate_field(index, view, window, cx);
            },
        ))
}

fn activate_field(
    index: usize,
    view: &mut ExplorerView,
    window: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let Some(server_index): Option<usize> = view.harness.selected else {
        return;
    };
    let Some(tool_index): Option<usize> = view.harness.tool else {
        return;
    };
    let Some(field): Option<&mut FormField> = view
        .harness
        .servers
        .get_mut(server_index)
        .and_then(|server: &mut Server| server.tools.get_mut(tool_index))
        .and_then(|tool: &mut ToolView| tool.fields.get_mut(index))
    else {
        return;
    };
    match &field.kind {
        FieldKind::Boolean => field.value = (field.value != "true").to_string(),
        FieldKind::Enum(options) => {
            let labels: Vec<String> = options.iter().map(enum_label).collect();
            let next: usize = labels
                .iter()
                .position(|label: &String| label == &field.value)
                .map_or(0, |position: usize| {
                    position.saturating_add(1) % labels.len().max(1)
                });
            if let Some(label) = labels.get(next) {
                field.value.clone_from(label);
            }
        }
        FieldKind::String
        | FieldKind::Integer
        | FieldKind::Number
        | FieldKind::Array(_)
        | FieldKind::Json => view.harness.edit = Some(index),
    }
    window.focus(&view.harness_focus, cx);
    cx.notify();
}

pub(crate) fn key_down(
    view: &mut ExplorerView,
    event: &gpui::KeyDownEvent,
    _: &mut Window,
    cx: &mut Context<ExplorerView>,
) {
    let key: &str = event.keystroke.key.as_str();
    if view.harness.search_edit {
        edit_text(
            &mut view.harness.search,
            key,
            event.keystroke.key_char.as_deref(),
        );
    } else if let (Some(server_index), Some(tool_index), Some(field_index)) =
        (view.harness.selected, view.harness.tool, view.harness.edit)
    {
        if let Some(field) = view
            .harness
            .servers
            .get_mut(server_index)
            .and_then(|server: &mut Server| server.tools.get_mut(tool_index))
            .and_then(|tool: &mut ToolView| tool.fields.get_mut(field_index))
        {
            edit_text(&mut field.value, key, event.keystroke.key_char.as_deref());
        }
    }
    if key == "enter" {
        view.harness.edit = None;
        view.harness.search_edit = false;
    }
    cx.stop_propagation();
    cx.notify();
}

fn edit_text(value: &mut String, key: &str, typed: Option<&str>) {
    match key {
        "backspace" => {
            value.pop();
        }
        "enter" => {}
        _ => {
            if let Some(text) = typed {
                value.push_str(text);
            }
        }
    }
}
