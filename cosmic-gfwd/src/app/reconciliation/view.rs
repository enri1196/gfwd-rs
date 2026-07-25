use cosmic::widget::{self, button, settings};

use crate::core::{
    CollectionValue, ScalarValue, ZoneReconciliationState, ZoneSetting, ZoneSettingDifference,
};
use crate::fl;

use crate::app::zones::ZoneViewAction;

use super::model::{
    ReconciliationPresentation, ReconciliationPresentationStatus, UnknownSettingNames,
};

/// User actions originating from reconciliation views.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ReconciliationAction {
    /// Open the detailed comparison drawer.
    Review,
    /// Reload the selected zone's permanent and runtime snapshots.
    Refresh,
    /// Request confirmation for applying permanent state globally.
    ApplyPermanentToRuntime,
    /// Request confirmation for saving runtime state globally.
    SaveRuntimeAsPermanent,
}

/// Build the selected-zone permanent/runtime reconciliation review content.
pub fn reconciliation_drawer<'a, Message: Clone + 'static>(
    state: &'a ZoneReconciliationState,
    mutation_pending: bool,
    operation_error: Option<&'a str>,
    watch_warning: Option<&'a str>,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let presentation = ReconciliationPresentation::from_state(state, mutation_pending);
    let refresh = button::standard(fl!("reconciliation-refresh")).on_press_maybe(
        presentation
            .actions
            .can_refresh
            .then_some(map(ZoneViewAction::Reconciliation(
                ReconciliationAction::Refresh,
            ))),
    );
    let mut status = settings::section()
        .title(fl!("reconciliation-status-heading"))
        .add(
            settings::item::builder(reconciliation_status(presentation.status, state))
                .description(fl!("reconciliation-refresh-description"))
                .control(refresh),
        );
    if let Some(error) = operation_error {
        status = status.add(
            settings::item::builder(fl!("reconciliation-operation-error-title"))
                .control(widget::text::body(error)),
        );
    }
    if let Some(error) = watch_warning {
        status = status.add(
            settings::item::builder(fl!("reconciliation-watch-warning-title")).control(
                widget::text::body(fl!("reconciliation-watch-warning", error = error)),
            ),
        );
    }
    if matches!(
        presentation.status,
        ReconciliationPresentationStatus::Incomplete { .. }
    ) {
        status = status
            .add(
                settings::item::builder(fl!("reconciliation-incomplete-warning-title")).control(
                    widget::text::body(fl!("reconciliation-unknown-explanation")),
                ),
            )
            .add(
                settings::item::builder(fl!("reconciliation-unknown-setting"))
                    .control(unknown_setting_values(&presentation.unknown_settings)),
            );
    }

    let mut content = widget::column::with_capacity(3)
        .push(status)
        .spacing(spacing.space_m);

    let mut differences = settings::section().title(fl!("reconciliation-differences-heading"));
    if presentation.differences.scalar.is_empty() && presentation.differences.collection.is_empty()
    {
        differences = differences.add(widget::text::body(fl!("reconciliation-no-differences")));
    } else {
        if !presentation.differences.scalar.is_empty() {
            differences = differences.add(difference_section(
                fl!("reconciliation-group-scalars"),
                &presentation.differences.scalar,
            ));
        }
        if !presentation.differences.collection.is_empty() {
            differences = differences.add(difference_section(
                fl!("reconciliation-group-collections"),
                &presentation.differences.collection,
            ));
        }
    }
    content = content.push(differences);

    let apply =
        button::destructive(fl!("reconciliation-apply-permanent")).on_press_maybe(
            presentation.actions.can_apply_permanent.then_some(map(
                ZoneViewAction::Reconciliation(ReconciliationAction::ApplyPermanentToRuntime),
            )),
        );
    let save = button::destructive(fl!("reconciliation-save-runtime")).on_press_maybe(
        presentation
            .actions
            .can_save_runtime
            .then_some(map(ZoneViewAction::Reconciliation(
                ReconciliationAction::SaveRuntimeAsPermanent,
            ))),
    );
    let global_actions = settings::section()
        .title(fl!("reconciliation-global-actions-heading"))
        .add(
            settings::item::builder(fl!("reconciliation-apply-permanent"))
                .description(fl!("reconciliation-apply-permanent-explanation"))
                .control(apply),
        )
        .add(
            settings::item::builder(fl!("reconciliation-save-runtime"))
                .description(fl!("reconciliation-save-runtime-explanation"))
                .control(save),
        );

    content.push(global_actions).into()
}

fn difference_section<'a, Message: Clone + 'static>(
    title: String,
    differences: &[&ZoneSettingDifference],
) -> cosmic::Element<'a, Message> {
    differences
        .iter()
        .fold(settings::section().title(title), |section, difference| {
            let (setting, permanent, runtime) = difference_values(difference);
            section.add(
                settings::item::builder(setting_label(setting))
                    .control(comparison_values(permanent, runtime)),
            )
        })
        .into()
}

fn comparison_values<'a, Message: Clone + 'static>(
    permanent: String,
    runtime: String,
) -> cosmic::Element<'a, Message> {
    widget::column::with_capacity(2)
        .push(labeled_value(
            fl!("reconciliation-permanent-value-label"),
            permanent,
        ))
        .push(labeled_value(
            fl!("reconciliation-runtime-value-label"),
            runtime,
        ))
        .spacing(cosmic::theme::spacing().space_s)
        .into()
}

fn labeled_value<'a, Message: Clone + 'static>(
    label: String,
    value: String,
) -> cosmic::Element<'a, Message> {
    widget::column::with_capacity(2)
        .push(widget::text::caption(label))
        .push(widget::text::body(value))
        .into()
}

fn unknown_setting_values<'a, Message: Clone + 'static>(
    unknown: &UnknownSettingNames<'_>,
) -> cosmic::Element<'a, Message> {
    let permanent = list_values(&unknown.permanent);
    let runtime = list_values(&unknown.runtime);
    comparison_values(permanent, runtime)
}

fn list_values(values: &[&str]) -> String {
    if values.is_empty() {
        fl!("reconciliation-none")
    } else {
        values.join("\n")
    }
}

fn difference_values(difference: &ZoneSettingDifference) -> (ZoneSetting, String, String) {
    match difference {
        ZoneSettingDifference::Scalar {
            setting,
            permanent,
            runtime,
        } => (*setting, scalar_value(permanent), scalar_value(runtime)),
        ZoneSettingDifference::Collection {
            setting,
            permanent_only,
            runtime_only,
        } => (
            *setting,
            collection_values(permanent_only),
            collection_values(runtime_only),
        ),
    }
}

fn scalar_value(value: &ScalarValue) -> String {
    match value {
        ScalarValue::Text(value) if value.is_empty() => fl!("reconciliation-empty-value"),
        ScalarValue::Text(value) => value.clone(),
        ScalarValue::Boolean(true) => fl!("reconciliation-yes"),
        ScalarValue::Boolean(false) => fl!("reconciliation-no"),
        ScalarValue::Integer(value) => value.to_string(),
    }
}

fn collection_values(values: &[CollectionValue]) -> String {
    if values.is_empty() {
        return fl!("reconciliation-absent-value");
    }
    values
        .iter()
        .map(|value| match value {
            CollectionValue::Text(value) => value.clone(),
            CollectionValue::Pair(first, second) => format!("{first}/{second}"),
            CollectionValue::ForwardPort(port, protocol, to_port, to_address) => {
                format!("{port}/{protocol} -> {to_port} ({to_address})")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn reconciliation_status(
    status: ReconciliationPresentationStatus,
    state: &ZoneReconciliationState,
) -> String {
    match status {
        ReconciliationPresentationStatus::Loading => fl!("reconciliation-status-loading"),
        ReconciliationPresentationStatus::Unavailable => {
            fl!("reconciliation-status-unavailable")
        }
        ReconciliationPresentationStatus::Error => {
            let message = match state {
                ZoneReconciliationState::Error { message, .. } => message.as_str(),
                _ => "",
            };
            fl!("reconciliation-status-error", error = message)
        }
        ReconciliationPresentationStatus::InSync => fl!("reconciliation-status-in-sync"),
        ReconciliationPresentationStatus::Different { count } => {
            fl!("reconciliation-status-different", count = count)
        }
        ReconciliationPresentationStatus::Incomplete {
            known_difference_count,
        } => fl!(
            "reconciliation-status-incomplete",
            count = known_difference_count
        ),
    }
}

fn setting_label(setting: ZoneSetting) -> String {
    match setting {
        ZoneSetting::ShortName => fl!("reconciliation-setting-short-name"),
        ZoneSetting::Description => fl!("reconciliation-setting-description"),
        ZoneSetting::Target => fl!("reconciliation-setting-target"),
        ZoneSetting::Services => fl!("reconciliation-setting-services"),
        ZoneSetting::Ports => fl!("reconciliation-setting-ports"),
        ZoneSetting::Protocols => fl!("reconciliation-setting-protocols"),
        ZoneSetting::SourcePorts => fl!("reconciliation-setting-source-ports"),
        ZoneSetting::IcmpBlocks => fl!("reconciliation-setting-icmp-blocks"),
        ZoneSetting::IcmpBlockInversion => fl!("reconciliation-setting-icmp-inversion"),
        ZoneSetting::Masquerade => fl!("reconciliation-setting-masquerade"),
        ZoneSetting::ForwardPorts => fl!("reconciliation-setting-forward-ports"),
        ZoneSetting::Interfaces => fl!("reconciliation-setting-interfaces"),
        ZoneSetting::Sources => fl!("reconciliation-setting-sources"),
        ZoneSetting::RichRules => fl!("reconciliation-setting-rich-rules"),
        ZoneSetting::Forward => fl!("reconciliation-setting-forward"),
        ZoneSetting::IngressPriority => fl!("reconciliation-setting-ingress-priority"),
        ZoneSetting::EgressPriority => fl!("reconciliation-setting-egress-priority"),
    }
}
