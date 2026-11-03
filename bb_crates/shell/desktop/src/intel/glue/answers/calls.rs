//! The callers answer (BBEOD24): every call site in the places panel, even
//! a single one (callers are a list to read, not a jump); none says so.

use crate::intel::bb_worker::protocol::{AskKind, Place};
use crate::intel::model::popups::{Pick, PlacesPanel};
use crate::ExplorerView;

/// Show the call sites of `name`, made by `callers` symbols.
pub(super) fn show(view: &mut ExplorerView, name: &str, callers: usize, places: Vec<Place>) {
    if places.is_empty() {
        view.report(&super::not_found(AskKind::Callers));
        return;
    }
    view.intel.references = Some(PlacesPanel {
        title: title(name, callers, places.len()),
        places,
        pick: Pick::default(),
    });
}

/// `2 callers of Hello`; when a caller calls more than once, the calls are
/// counted too: `3 calls from 2 callers of Hello`.
fn title(name: &str, callers: usize, calls: usize) -> String {
    let who: String = if callers == 1 {
        String::from("1 caller")
    } else {
        format!("{callers} callers")
    };
    if calls == callers {
        format!("{who} of {name}")
    } else {
        format!("{calls} calls from {who} of {name}")
    }
}

#[cfg(test)]
mod tests {
    use super::title;

    #[test]
    fn the_title_counts_callers_and_names_more_calls_than_callers() {
        assert_eq!(title("Hello", 1, 1), "1 caller of Hello");
        assert_eq!(title("Hello", 2, 2), "2 callers of Hello");
        assert_eq!(title("Hello", 1, 3), "3 calls from 1 caller of Hello");
    }
}
