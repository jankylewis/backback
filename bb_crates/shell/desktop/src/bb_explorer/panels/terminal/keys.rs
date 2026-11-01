//! A keystroke as the bytes a shell expects (xterm sequences).

/// The modifiers that change what a key sends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Mods {
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) cmd: bool,
}

/// Bytes for `key` (gpui key name) and its typed text; `app_cursor` is the
/// terminal's DECCKM mode (arrows send `ESC O x` instead of `ESC [ x`).
/// `None` for keys the terminal leaves to the app (Cmd shortcuts).
pub(crate) fn bytes(
    key: &str,
    text: Option<&str>,
    mods: Mods,
    app_cursor: bool,
) -> Option<Vec<u8>> {
    if mods.cmd {
        return None;
    }
    let fixed: Option<&[u8]> = match key {
        "enter" => Some(b"\r"),
        "backspace" => Some(b"\x7f"),
        "tab" => Some(b"\t"),
        "escape" => Some(b"\x1b"),
        "delete" => Some(b"\x1b[3~"),
        "home" => Some(b"\x1b[H"),
        "end" => Some(b"\x1b[F"),
        "pageup" => Some(b"\x1b[5~"),
        "pagedown" => Some(b"\x1b[6~"),
        _ => None,
    };
    if let Some(bytes) = fixed {
        return Some(bytes.to_vec());
    }
    match key {
        "up" => return Some(arrow(app_cursor, 'A')),
        "down" => return Some(arrow(app_cursor, 'B')),
        "right" => return Some(arrow(app_cursor, 'C')),
        "left" => return Some(arrow(app_cursor, 'D')),
        _ => {}
    }
    if mods.ctrl {
        let letter: char = key
            .chars()
            .next()
            .filter(|_: &char| key.chars().count() == 1)?;
        let code: u8 = u8::try_from(letter.to_ascii_lowercase()).ok()?;
        return code.is_ascii_lowercase().then(|| vec![code & 0x1f]);
    }
    let text: &str = text?;
    let mut out: Vec<u8> = Vec::new();
    if mods.alt {
        out.push(0x1b);
    }
    out.extend_from_slice(text.as_bytes());
    Some(out)
}

/// An arrow key: `ESC [ x`, or `ESC O x` in application cursor mode.
fn arrow(app_cursor: bool, letter: char) -> Vec<u8> {
    let lead: &str = if app_cursor { "\x1bO" } else { "\x1b[" };
    format!("{lead}{letter}").into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Mods = Mods {
        ctrl: false,
        alt: false,
        cmd: false,
    };

    #[test]
    fn text_enter_backspace_and_arrows_are_xterm_bytes() {
        assert_eq!(bytes("a", Some("a"), NONE, false), Some(b"a".to_vec()));
        assert_eq!(bytes("enter", None, NONE, false), Some(b"\r".to_vec()));
        assert_eq!(bytes("backspace", None, NONE, false), Some(vec![0x7f]));
        assert_eq!(bytes("up", None, NONE, false), Some(b"\x1b[A".to_vec()));
        assert_eq!(
            bytes("up", None, NONE, true),
            Some(b"\x1bOA".to_vec()),
            "app cursor mode"
        );
    }

    #[test]
    fn ctrl_letters_are_control_codes_and_cmd_is_left_to_the_app() {
        let ctrl: Mods = Mods { ctrl: true, ..NONE };
        assert_eq!(bytes("c", Some("c"), ctrl, false), Some(vec![0x03]));
        assert_eq!(bytes("d", Some("d"), ctrl, false), Some(vec![0x04]));
        let cmd: Mods = Mods { cmd: true, ..NONE };
        assert_eq!(bytes("v", Some("v"), cmd, false), None);
        let alt: Mods = Mods { alt: true, ..NONE };
        assert_eq!(bytes("b", Some("b"), alt, false), Some(b"\x1bb".to_vec()));
    }
}
