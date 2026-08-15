use cosmic::iced::Length;
use cosmic::widget::{self, dropdown, settings};

use crate::fl;
use crate::models::ZoneTarget;

use super::DialogMessage;

#[derive(Debug, Clone)]
pub struct ZoneFormState {
    pub name: String,
    pub description: String,
    pub target: ZoneTarget,
}

impl Default for ZoneFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            target: ZoneTarget::Default,
        }
    }
}

pub(super) fn name_changed(state: &mut ZoneFormState, value: String) {
    state.name = value;
}

pub(super) fn description_changed(state: &mut ZoneFormState, value: String) {
    state.description = value;
}

pub(super) fn target_selected(state: &mut ZoneFormState, index: usize) {
    state.target = target_from_index(index);
}

pub fn zone_drawer<'a>(state: &'a ZoneFormState) -> cosmic::Element<'a, DialogMessage> {
    let target_labels = target_labels();
    let target_selected = Some(target_index(&state.target));

    let content = settings::view_column(vec![
        settings::section()
            .title(fl!("dialog-zone-section-basic"))
            .add(
                settings::item::builder(fl!("dialog-zone-name-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-zone-name-placeholder"),
                        &state.name,
                    )
                    .on_input(DialogMessage::ZoneNameChanged)
                    .width(Length::Fill),
                ),
            )
            .add(
                settings::item::builder(fl!("dialog-zone-description-label")).control(
                    widget::text_input::text_input(
                        fl!("dialog-zone-description-placeholder"),
                        &state.description,
                    )
                    .on_input(DialogMessage::ZoneDescriptionChanged)
                    .width(Length::Fill),
                ),
            )
            .into(),
        settings::section()
            .title(fl!("dialog-zone-section-target"))
            .add(
                settings::item::builder(fl!("dialog-zone-target-label")).control(
                    dropdown(
                        target_labels,
                        target_selected,
                        DialogMessage::ZoneTargetSelected,
                    )
                    .width(Length::Fill),
                ),
            )
            .into(),
    ]);

    content.into()
}

pub fn target_from_index(index: usize) -> ZoneTarget {
    match index {
        1 => ZoneTarget::Accept,
        2 => ZoneTarget::Drop,
        3 => ZoneTarget::Reject,
        _ => ZoneTarget::Default,
    }
}

pub fn target_index(target: &ZoneTarget) -> usize {
    match target {
        ZoneTarget::Default => 0,
        ZoneTarget::Accept => 1,
        ZoneTarget::Drop => 2,
        ZoneTarget::Reject => 3,
        ZoneTarget::Other(_) => 0,
    }
}

fn target_labels() -> Vec<String> {
    vec![
        fl!("dialog-target-default"),
        fl!("dialog-target-accept"),
        fl!("dialog-target-drop"),
        fl!("dialog-target-reject"),
    ]
}
