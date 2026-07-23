use cosmic::iced::Length;
use cosmic::widget::{self, button, settings};

use crate::core::{
    CollectionValue, ComparisonCompleteness, ScalarValue, ZoneReconciliationState, ZoneSetting,
    ZoneSettingDifference,
};
use crate::fl;

use super::ZoneViewAction;

/// Build the selected-zone permanent/runtime reconciliation review content.
pub fn reconciliation_drawer<'a, Message: Clone + 'static>(
    state: &'a ZoneReconciliationState,
    mutation_pending: bool,
    operation_error: Option<&'a str>,
    map: impl Fn(ZoneViewAction) -> Message + Copy + 'static,
) -> cosmic::Element<'a, Message> {
    let spacing = cosmic::theme::spacing();
    let has_differences = state
        .data()
        .is_some_and(|data| !data.reconciliation.differences.is_empty());
    let has_runtime_only = state
        .data()
        .is_some_and(|data| data.reconciliation.has_runtime_only_differences());
    let actions = widget::row::with_capacity(3)
        .push(
            button::standard(fl!("reconciliation-refresh")).on_press_maybe(
                (!mutation_pending).then_some(map(ZoneViewAction::RefreshReconciliation)),
            ),
        )
        .push(
            button::destructive(fl!("reconciliation-apply-permanent")).on_press_maybe(
                (has_differences && !mutation_pending)
                    .then_some(map(ZoneViewAction::ApplyPermanentConfiguration)),
            ),
        )
        .push(
            button::destructive(fl!("reconciliation-save-runtime")).on_press_maybe(
                (has_runtime_only && !mutation_pending)
                    .then_some(map(ZoneViewAction::SaveRuntimeConfiguration)),
            ),
        )
        .spacing(spacing.space_s);
    let mut content = widget::column::with_capacity(7)
        .push(widget::text::body(fl!("reconciliation-review-description")))
        .push(actions)
        .spacing(spacing.space_m);
    if let Some(error) = operation_error {
        content = content.push(widget::text::body(error));
    }

    let Some(data) = state.data() else {
        return content
            .push(widget::text::body(reconciliation_status(state)))
            .into();
    };

    let scalar_differences: Vec<_> = data
        .reconciliation
        .differences
        .iter()
        .filter(|difference| matches!(difference, ZoneSettingDifference::Scalar { .. }))
        .collect();
    let collection_differences: Vec<_> = data
        .reconciliation
        .differences
        .iter()
        .filter(|difference| matches!(difference, ZoneSettingDifference::Collection { .. }))
        .collect();

    if scalar_differences.is_empty() && collection_differences.is_empty() {
        content = content.push(widget::text::body(fl!("reconciliation-no-differences")));
    }
    if !scalar_differences.is_empty() {
        content = content.push(difference_section(
            fl!("reconciliation-group-scalars"),
            &scalar_differences,
        ));
    }
    if !collection_differences.is_empty() {
        content = content.push(difference_section(
            fl!("reconciliation-group-collections"),
            &collection_differences,
        ));
    }

    if let ComparisonCompleteness::Incomplete {
        permanent_unknown,
        runtime_unknown,
    } = &data.reconciliation.completeness
    {
        let permanent = if permanent_unknown.is_empty() {
            fl!("reconciliation-none")
        } else {
            permanent_unknown
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        };
        let runtime = if runtime_unknown.is_empty() {
            fl!("reconciliation-none")
        } else {
            runtime_unknown
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        };
        content = content
            .push(widget::text::body(fl!(
                "reconciliation-unknown-explanation"
            )))
            .push(
                settings::section()
                    .title(fl!("reconciliation-unknown-title"))
                    .add(column_header())
                    .add(
                        settings::item::builder(fl!("reconciliation-unknown-setting"))
                            .control(comparison_columns(permanent, runtime)),
                    ),
            );
    }

    content.into()
}

fn difference_section<'a, Message: Clone + 'static>(
    title: String,
    differences: &[&ZoneSettingDifference],
) -> cosmic::Element<'a, Message> {
    let section = differences.iter().fold(
        settings::section().title(title).add(column_header()),
        |section, difference| {
            let (setting, permanent, runtime) = difference_values(difference);
            section.add(
                settings::item::builder(setting_label(setting))
                    .control(comparison_columns(permanent, runtime)),
            )
        },
    );
    section.into()
}

fn column_header<'a, Message: Clone + 'static>() -> cosmic::Element<'a, Message> {
    settings::item::builder(fl!("reconciliation-setting-column"))
        .control(comparison_columns(
            fl!("reconciliation-permanent-column"),
            fl!("reconciliation-runtime-column"),
        ))
        .into()
}

fn comparison_columns<'a, Message: Clone + 'static>(
    permanent: String,
    runtime: String,
) -> cosmic::Element<'a, Message> {
    let column_width = Length::Fixed(170.0);
    widget::row::with_capacity(2)
        .push(
            widget::container(widget::text::body(permanent))
                .width(column_width)
                .padding([0, cosmic::theme::spacing().space_s]),
        )
        .push(
            widget::container(widget::text::body(runtime))
                .width(column_width)
                .padding([0, cosmic::theme::spacing().space_s]),
        )
        .into()
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
        return fl!("reconciliation-none");
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

fn reconciliation_status(state: &ZoneReconciliationState) -> String {
    match state {
        ZoneReconciliationState::Loading { .. } => fl!("reconciliation-status-loading"),
        ZoneReconciliationState::Unavailable { .. } => fl!("reconciliation-status-unavailable"),
        ZoneReconciliationState::Error { message, .. } => {
            fl!("reconciliation-status-error", error = message)
        }
        ZoneReconciliationState::InSync { .. } => fl!("reconciliation-status-in-sync"),
        ZoneReconciliationState::Different { data, .. } => fl!(
            "reconciliation-status-different",
            count = data.reconciliation.differences.len()
        ),
        ZoneReconciliationState::Incomplete { data, .. } => fl!(
            "reconciliation-status-incomplete",
            count = data.reconciliation.differences.len()
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
