//! Session setup order shared by launch and attach.

use crate::DapError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SessionState {
    Created,
    Initialized,
    Configuring,
    Running,
}

impl SessionState {
    pub(super) fn next(self, action: Action) -> Result<Self, DapError> {
        match (self, action) {
            (Self::Created, Action::Initialize) => Ok(Self::Initialized),
            (Self::Initialized, Action::Launch | Action::Attach) => Ok(Self::Configuring),
            (Self::Configuring, Action::ConfigurationDone) => Ok(Self::Running),
            _ => Err(DapError::State),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Action {
    Initialize,
    Launch,
    Attach,
    ConfigurationDone,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_order_rejects_missing_and_repeated_steps() {
        assert!(SessionState::Created.next(Action::Launch).is_err());
        let initialized: SessionState = SessionState::Created.next(Action::Initialize).unwrap();
        assert!(initialized.next(Action::Initialize).is_err());
        let configuring: SessionState = initialized.next(Action::Attach).unwrap();
        assert!(configuring.next(Action::Launch).is_err());
        assert_eq!(
            configuring.next(Action::ConfigurationDone).unwrap(),
            SessionState::Running
        );
    }
}
