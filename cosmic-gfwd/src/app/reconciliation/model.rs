//! Semantic presentation state for this slice's views.
//!
//! This module deliberately contains no localization or widget code. Both the
//! selected-zone banner and the review drawer use this adapter so status,
//! grouping, and action availability are computed in one place.

use crate::core::{ComparisonCompleteness, ZoneReconciliationState, ZoneSettingDifference};

/// Localization-independent reconciliation state consumed by the UI.
#[derive(Debug)]
pub(crate) struct ReconciliationPresentation<'a> {
    /// Semantic status of the current comparison.
    pub(crate) status: ReconciliationPresentationStatus,
    /// Actions that the current state permits.
    pub(crate) actions: ReconciliationActionAvailability,
    /// Known differences grouped for display.
    pub(crate) differences: ReconciliationDifferenceGroups<'a>,
    /// Unknown dictionary keys that made the comparison incomplete.
    pub(crate) unknown_settings: UnknownSettingNames<'a>,
}

/// Semantic status of the selected-zone comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReconciliationPresentationStatus {
    /// A comparison request is in flight.
    Loading,
    /// All known settings match and the comparison was complete.
    InSync,
    /// A complete comparison found known differences.
    Different {
        /// Number of known setting differences.
        count: usize,
    },
    /// Unknown keys prevent a definitive result.
    Incomplete {
        /// Number of differences among settings understood by the application.
        known_difference_count: usize,
    },
    /// Runtime comparison is currently unavailable.
    Unavailable,
    /// The comparison request failed.
    Error,
}

/// Reconciliation actions permitted by the current state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ReconciliationActionAvailability {
    /// Whether the detailed comparison contains meaningful information.
    pub(crate) can_review: bool,
    /// Whether a manual comparison refresh can be started.
    pub(crate) can_refresh: bool,
    /// Whether permanent configuration can be applied to runtime.
    pub(crate) can_apply_permanent: bool,
    /// Whether runtime configuration can be persisted permanently.
    pub(crate) can_save_runtime: bool,
}

/// Known setting differences grouped by their display shape.
#[derive(Debug, Default)]
pub(crate) struct ReconciliationDifferenceGroups<'a> {
    /// Differences whose permanent and runtime sides are single values.
    pub(crate) scalar: Vec<&'a ZoneSettingDifference>,
    /// Differences whose sides contain order-independent collections.
    pub(crate) collection: Vec<&'a ZoneSettingDifference>,
}

/// Unknown keys reported independently by permanent and runtime snapshots.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct UnknownSettingNames<'a> {
    /// Unknown permanent configuration keys, in stable sorted order.
    pub(crate) permanent: Vec<&'a str>,
    /// Unknown runtime configuration keys, in stable sorted order.
    pub(crate) runtime: Vec<&'a str>,
}

impl ReconciliationPresentation<'_> {
    /// Adapt domain reconciliation state for rendering without cloning snapshots.
    pub(crate) fn from_state(
        state: &ZoneReconciliationState,
        mutation_pending: bool,
    ) -> ReconciliationPresentation<'_> {
        let mut differences = ReconciliationDifferenceGroups::default();
        let mut unknown_settings = UnknownSettingNames::default();

        if let Some(data) = state.data() {
            for difference in &data.reconciliation.differences {
                match difference {
                    ZoneSettingDifference::Scalar { .. } => {
                        differences.scalar.push(difference);
                    }
                    ZoneSettingDifference::Collection { .. } => {
                        differences.collection.push(difference);
                    }
                }
            }

            if let ComparisonCompleteness::Incomplete {
                permanent_unknown,
                runtime_unknown,
            } = &data.reconciliation.completeness
            {
                unknown_settings.permanent = permanent_unknown.iter().map(String::as_str).collect();
                unknown_settings.runtime = runtime_unknown.iter().map(String::as_str).collect();
            }
        }

        let known_difference_count = differences.scalar.len() + differences.collection.len();
        let status = match state {
            ZoneReconciliationState::Loading { .. } => ReconciliationPresentationStatus::Loading,
            ZoneReconciliationState::InSync { .. } => ReconciliationPresentationStatus::InSync,
            ZoneReconciliationState::Different { .. } => {
                ReconciliationPresentationStatus::Different {
                    count: known_difference_count,
                }
            }
            ZoneReconciliationState::Incomplete { .. } => {
                ReconciliationPresentationStatus::Incomplete {
                    known_difference_count,
                }
            }
            ZoneReconciliationState::Unavailable { .. } => {
                ReconciliationPresentationStatus::Unavailable
            }
            ZoneReconciliationState::Error { .. } => ReconciliationPresentationStatus::Error,
        };

        let has_runtime_only = state
            .data()
            .is_some_and(|data| data.reconciliation.has_runtime_only_differences());
        let is_loading = matches!(state, ZoneReconciliationState::Loading { .. });
        let can_review = matches!(
            state,
            ZoneReconciliationState::Different { .. } | ZoneReconciliationState::Incomplete { .. }
        );

        ReconciliationPresentation {
            status,
            actions: ReconciliationActionAvailability {
                can_review: can_review && !mutation_pending,
                can_refresh: !is_loading && !mutation_pending,
                can_apply_permanent: known_difference_count > 0 && !mutation_pending && !is_loading,
                can_save_runtime: has_runtime_only && !mutation_pending && !is_loading,
            },
            differences,
            unknown_settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::core::{
        CollectionValue, ComparisonCompleteness, ScalarValue, ZoneReconciliationData,
        ZoneReconciliationState, ZoneSetting, ZoneSettingDifference,
        reconciliation::{ZoneReconciliation, ZoneSettingsSnapshot},
    };

    use super::{ReconciliationPresentation, ReconciliationPresentationStatus};

    fn data(
        differences: Vec<ZoneSettingDifference>,
        completeness: ComparisonCompleteness,
    ) -> ZoneReconciliationData {
        ZoneReconciliationData {
            permanent: ZoneSettingsSnapshot::default(),
            runtime: ZoneSettingsSnapshot::default(),
            reconciliation: ZoneReconciliation {
                differences,
                completeness,
            },
        }
    }

    fn scalar_difference() -> ZoneSettingDifference {
        ZoneSettingDifference::Scalar {
            setting: ZoneSetting::Target,
            permanent: ScalarValue::Text("DROP".into()),
            runtime: ScalarValue::Text("ACCEPT".into()),
        }
    }

    fn collection_difference(
        permanent_only: Vec<CollectionValue>,
        runtime_only: Vec<CollectionValue>,
    ) -> ZoneSettingDifference {
        ZoneSettingDifference::Collection {
            setting: ZoneSetting::Services,
            permanent_only,
            runtime_only,
        }
    }

    #[test]
    fn loading_disables_every_action() {
        let state = ZoneReconciliationState::Loading {
            zone: "public".into(),
        };

        let presentation = ReconciliationPresentation::from_state(&state, false);

        assert_eq!(
            presentation.status,
            ReconciliationPresentationStatus::Loading
        );
        assert_eq!(presentation.differences.scalar.len(), 0);
        assert!(!presentation.actions.can_review);
        assert!(!presentation.actions.can_refresh);
        assert!(!presentation.actions.can_apply_permanent);
        assert!(!presentation.actions.can_save_runtime);
    }

    #[test]
    fn in_sync_is_refreshable_without_destructive_actions() {
        let state = ZoneReconciliationState::from_data(
            "public".into(),
            data(Vec::new(), ComparisonCompleteness::Complete),
        );

        let presentation = ReconciliationPresentation::from_state(&state, false);

        assert_eq!(
            presentation.status,
            ReconciliationPresentationStatus::InSync
        );
        assert!(presentation.actions.can_refresh);
        assert!(!presentation.actions.can_review);
        assert!(!presentation.actions.can_apply_permanent);
        assert!(!presentation.actions.can_save_runtime);
    }

    #[test]
    fn known_differences_are_reviewable_and_applicable() {
        let state = ZoneReconciliationState::from_data(
            "public".into(),
            data(vec![scalar_difference()], ComparisonCompleteness::Complete),
        );

        let presentation = ReconciliationPresentation::from_state(&state, false);

        assert_eq!(
            presentation.status,
            ReconciliationPresentationStatus::Different { count: 1 }
        );
        assert!(presentation.actions.can_review);
        assert!(presentation.actions.can_apply_permanent);
        assert!(presentation.actions.can_save_runtime);
    }

    #[test]
    fn incomplete_comparison_exposes_known_and_unknown_settings() {
        let state = ZoneReconciliationState::from_data(
            "public".into(),
            data(
                vec![scalar_difference()],
                ComparisonCompleteness::Incomplete {
                    permanent_unknown: BTreeSet::from(["future-permanent".into()]),
                    runtime_unknown: BTreeSet::from(["future-runtime".into()]),
                },
            ),
        );

        let presentation = ReconciliationPresentation::from_state(&state, false);

        assert_eq!(
            presentation.status,
            ReconciliationPresentationStatus::Incomplete {
                known_difference_count: 1,
            }
        );
        assert_eq!(
            presentation.unknown_settings.permanent,
            ["future-permanent"]
        );
        assert_eq!(presentation.unknown_settings.runtime, ["future-runtime"]);
        assert!(presentation.actions.can_review);
    }

    #[test]
    fn incomplete_comparison_without_known_differences_is_never_in_sync() {
        let state = ZoneReconciliationState::from_data(
            "public".into(),
            data(
                Vec::new(),
                ComparisonCompleteness::Incomplete {
                    permanent_unknown: BTreeSet::from(["future-setting".into()]),
                    runtime_unknown: BTreeSet::new(),
                },
            ),
        );

        let presentation = ReconciliationPresentation::from_state(&state, false);

        assert_eq!(
            presentation.status,
            ReconciliationPresentationStatus::Incomplete {
                known_difference_count: 0,
            }
        );
        assert_ne!(
            presentation.status,
            ReconciliationPresentationStatus::InSync
        );
        assert!(presentation.actions.can_review);
        assert!(!presentation.actions.can_apply_permanent);
        assert!(!presentation.actions.can_save_runtime);
    }

    #[test]
    fn unavailable_runtime_remains_refreshable() {
        let state = ZoneReconciliationState::Unavailable {
            zone: Some("public".into()),
        };

        let presentation = ReconciliationPresentation::from_state(&state, false);

        assert_eq!(
            presentation.status,
            ReconciliationPresentationStatus::Unavailable
        );
        assert!(presentation.actions.can_refresh);
        assert!(!presentation.actions.can_review);
    }

    #[test]
    fn load_error_remains_refreshable() {
        let state = ZoneReconciliationState::Error {
            zone: "public".into(),
            message: "failed".into(),
        };

        let presentation = ReconciliationPresentation::from_state(&state, false);

        assert_eq!(presentation.status, ReconciliationPresentationStatus::Error);
        assert!(presentation.actions.can_refresh);
        assert!(!presentation.actions.can_review);
    }

    #[test]
    fn differences_are_grouped_without_cloning_snapshots() {
        let state = ZoneReconciliationState::from_data(
            "public".into(),
            data(
                vec![
                    scalar_difference(),
                    collection_difference(vec![CollectionValue::Text("ssh".into())], Vec::new()),
                ],
                ComparisonCompleteness::Complete,
            ),
        );

        let presentation = ReconciliationPresentation::from_state(&state, false);

        assert_eq!(presentation.differences.scalar.len(), 1);
        assert_eq!(presentation.differences.collection.len(), 1);
    }

    #[test]
    fn save_runtime_requires_a_runtime_only_difference() {
        let permanent_only = ZoneReconciliationState::from_data(
            "public".into(),
            data(
                vec![collection_difference(
                    vec![CollectionValue::Text("ssh".into())],
                    Vec::new(),
                )],
                ComparisonCompleteness::Complete,
            ),
        );
        let runtime_only = ZoneReconciliationState::from_data(
            "public".into(),
            data(
                vec![collection_difference(
                    Vec::new(),
                    vec![CollectionValue::Text("https".into())],
                )],
                ComparisonCompleteness::Complete,
            ),
        );

        assert!(
            !ReconciliationPresentation::from_state(&permanent_only, false)
                .actions
                .can_save_runtime
        );
        assert!(
            ReconciliationPresentation::from_state(&runtime_only, false)
                .actions
                .can_save_runtime
        );
    }

    #[test]
    fn pending_mutation_disables_all_actions() {
        let state = ZoneReconciliationState::from_data(
            "public".into(),
            data(vec![scalar_difference()], ComparisonCompleteness::Complete),
        );

        let actions = ReconciliationPresentation::from_state(&state, true).actions;

        assert!(!actions.can_review);
        assert!(!actions.can_refresh);
        assert!(!actions.can_apply_permanent);
        assert!(!actions.can_save_runtime);
    }
}
