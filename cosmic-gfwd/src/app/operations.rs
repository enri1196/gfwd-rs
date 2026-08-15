//! Root-owned mutation, confirmation, reload, and notification state.

use cosmic::widget::{Toast, ToastId, Toasts};

/// Cross-slice operation state that must remain globally serialized.
pub(crate) struct State<Message, Confirmation> {
    /// User-visible notifications for completed operations.
    pub(crate) toasts: Toasts<Message>,
    /// Name of the mutation currently in flight.
    pub(crate) pending: Option<String>,
    /// Destructive operation awaiting explicit confirmation.
    pub(crate) confirmation: Option<Confirmation>,
    /// Permanent configuration changed since the last explicit runtime reload.
    pub(crate) runtime_reload_needed: bool,
}

impl<Message: Clone + Send + 'static, Confirmation> State<Message, Confirmation> {
    /// Create operation state with the message used to dismiss notifications.
    pub(crate) fn new(dismiss: fn(ToastId) -> Message) -> Self {
        Self {
            toasts: Toasts::new(dismiss),
            pending: None,
            confirmation: None,
            runtime_reload_needed: false,
        }
    }

    /// Return whether a mutation is currently active.
    pub(crate) fn mutation_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub(crate) fn pending_label(&self) -> Option<&str> {
        self.pending.as_deref()
    }

    /// Begin a mutation if no other slice owns the global mutation slot.
    pub(crate) fn begin(&mut self, operation: String) -> bool {
        if self.mutation_pending() {
            return false;
        }
        self.pending = Some(operation);
        true
    }

    /// Consume the current operation label, falling back to the supplied label.
    pub(crate) fn finish_label(&mut self, fallback: String) -> String {
        self.pending.take().unwrap_or(fallback)
    }

    /// Add a user-visible notification.
    pub(crate) fn push_toast(&mut self, toast: Toast<Message>) -> cosmic::Task<Message> {
        self.toasts.push(toast)
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[derive(Clone)]
    enum Message {
        Dismiss,
    }

    #[test]
    fn only_one_mutation_can_be_active() {
        let mut state = State::<Message, ()>::new(|_| Message::Dismiss);

        assert!(state.begin("first".into()));
        assert!(!state.begin("second".into()));
        assert_eq!(state.finish_label("fallback".into()), "first");
        assert!(state.begin("second".into()));
    }

    #[test]
    fn confirmation_dispatch_is_consumed_once() {
        let mut state = State::<Message, &str>::new(|_| Message::Dismiss);
        state.confirmation = Some("delete");

        assert_eq!(state.confirmation.take(), Some("delete"));
        assert_eq!(state.confirmation.take(), None);
    }
}
