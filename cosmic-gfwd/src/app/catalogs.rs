//! Loading state shared by the application's typed option catalogs.

/// Items, progress, and failure state for one asynchronously loaded catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CatalogState<T> {
    items: Vec<T>,
    loading: bool,
    error: Option<String>,
}

impl<T> Default for CatalogState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            loading: false,
            error: None,
        }
    }
}

impl<T> CatalogState<T> {
    /// Begin loading while retaining existing items and clearing an old error.
    pub(crate) fn begin_load(&mut self) {
        self.loading = true;
        self.error = None;
    }

    /// Replace items after a successful load.
    pub(crate) fn finish(&mut self, items: Vec<T>) {
        self.items = items;
        self.loading = false;
        self.error = None;
    }

    /// Finish loading with an error and discard stale items.
    pub(crate) fn fail(&mut self, error: String) {
        self.items.clear();
        self.loading = false;
        self.error = Some(error);
    }

    /// Return the currently available items.
    pub(crate) fn items(&self) -> &[T] {
        &self.items
    }

    /// Return whether a load is currently in progress.
    pub(crate) fn is_loading(&self) -> bool {
        self.loading
    }

    /// Return the most recent loading error.
    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::CatalogState;

    #[test]
    fn begin_marks_loading_and_retains_previous_items() {
        let mut catalog = CatalogState::default();
        catalog.finish(vec!["old"]);

        catalog.begin_load();

        assert!(catalog.is_loading());
        assert_eq!(catalog.items(), ["old"]);
        assert_eq!(catalog.error(), None);
    }

    #[test]
    fn success_replaces_items_and_clears_loading() {
        let mut catalog = CatalogState::default();
        catalog.begin_load();

        catalog.finish(vec!["new"]);

        assert!(!catalog.is_loading());
        assert_eq!(catalog.items(), ["new"]);
        assert_eq!(catalog.error(), None);
    }

    #[test]
    fn failure_discards_items_and_exposes_exact_error() {
        let mut catalog = CatalogState::default();
        catalog.finish(vec!["cached"]);
        catalog.begin_load();

        catalog.fail("permission denied".into());

        assert!(!catalog.is_loading());
        assert!(catalog.items().is_empty());
        assert_eq!(catalog.error(), Some("permission denied"));
    }

    #[test]
    fn retry_clears_previous_error_without_discarding_items() {
        let mut catalog = CatalogState::default();
        catalog.finish(vec!["cached"]);
        catalog.fail("temporary failure".into());

        catalog.begin_load();

        assert!(catalog.is_loading());
        assert!(catalog.items().is_empty());
        assert_eq!(catalog.error(), None);
    }

    #[test]
    fn failure_can_be_replaced_by_later_success() {
        let mut catalog = CatalogState::default();
        catalog.fail("temporary failure".into());

        catalog.begin_load();
        catalog.finish(vec!["recovered"]);

        assert_eq!(catalog.items(), ["recovered"]);
        assert_eq!(catalog.error(), None);
    }
}
