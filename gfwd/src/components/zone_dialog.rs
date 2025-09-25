use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::fwd_broker::{ZoneSettings, ZoneTarget};
use crate::validation::validate_zone_name;

#[tracker::track]
#[derive(Debug)]
pub struct AddZoneDialog {
    name: String,
    description: String,
    target: ZoneTarget,
    name_valid: bool,
    name_error: Option<String>,
}

#[derive(Debug)]
pub enum AddZoneDialogRequest {
    SetName(String),
    SetDescription(String),
    SetTarget(ZoneTarget),
    ValidateName,
    Add,
    Cancel,
}

#[derive(Debug)]
pub enum AddZoneDialogResponse {
    ZoneSettings(ZoneSettings),
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AddZoneDialog {
    type Init = ();
    type Input = AddZoneDialogRequest;
    type Output = AddZoneDialogResponse;
    type Widgets = AddZoneDialogWidgets;

    view! {
        dialog = adw::Dialog {
            set_title: "Add Zone",

            #[wrap(Some)]
            set_child = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: "Zone Configuration",
                    set_description: Some("Configure the new firewall zone"),

                    // Name field
                    add = &adw::EntryRow {
                        set_title: "Zone Name",
                        #[track(model.changed(AddZoneDialog::name_error()))]
                        set_css_classes: if model.name_error.is_some() { &["error"] } else { &[] },
                        connect_changed[sender] => move |entry| {
                            sender.input(AddZoneDialogRequest::SetName(entry.text().to_string()));
                        },
                        connect_apply[sender] => move |_| {
                            sender.input(AddZoneDialogRequest::ValidateName);
                        },
                    },

                    // Name error label
                    add = &gtk::Label {
                        #[track(model.changed(AddZoneDialog::name_error()))]
                        set_text: model.name_error.as_deref().unwrap_or(""),
                        #[track(model.changed(AddZoneDialog::name_error()))]
                        set_visible: model.name_error.is_some(),
                        set_halign: gtk::Align::Start,
                        set_margin_start: 12,
                        add_css_class: "error",
                        add_css_class: "caption",
                    },

                    // Description field
                    add = &adw::EntryRow {
                        set_title: "Description",
                        connect_changed[sender] => move |entry| {
                            sender.input(AddZoneDialogRequest::SetDescription(entry.text().to_string()));
                        },
                    },

                    // Target selection
                    add = &adw::ComboRow {
                        set_title: "Default Target",
                        set_subtitle: "Action for packets not matching any rule",
                        set_model: Some(&gtk::StringList::new(&[
                            "default (use system default)",
                            "ACCEPT (allow all)",
                            "DROP (silently drop)",
                            "REJECT (reject with response)"
                        ])),
                        set_selected: match model.target {
                            ZoneTarget::Default => 0,
                            ZoneTarget::Accept => 1,
                            ZoneTarget::Drop => 2,
                            ZoneTarget::Reject => 3,
                        },
                        connect_selected_notify[sender] => move |combo| {
                            let target = match combo.selected() {
                                0 => ZoneTarget::Default,
                                1 => ZoneTarget::Accept,
                                2 => ZoneTarget::Drop,
                                3 => ZoneTarget::Reject,
                                _ => ZoneTarget::Default,
                            };
                            sender.input(AddZoneDialogRequest::SetTarget(target));
                        },
                    },
                },

                // Action buttons
                add = &adw::PreferencesGroup {
                    add = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_halign: gtk::Align::End,
                        set_spacing: 6,
                        set_margin_all: 12,

                        append = &gtk::Button::with_label("Cancel") {
                            connect_clicked[sender, root] => move |_| {
                                sender.input(AddZoneDialogRequest::Cancel);
                                root.close();
                            },
                        },

                        append = &gtk::Button::with_label("Add Zone") {
                            add_css_class: "suggested-action",
                            #[track(model.changed(AddZoneDialog::name_valid()))]
                            set_sensitive: model.name_valid && !model.name.is_empty(),
                            connect_clicked[sender, root] => move |_| {
                                sender.input(AddZoneDialogRequest::Add);
                                root.close();
                            },
                        },
                    },
                },
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AddZoneDialog {
            name: String::new(),
            description: String::new(),
            target: ZoneTarget::Default,
            name_valid: false,
            name_error: None,
            tracker: 0,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();
        
        match msg {
            AddZoneDialogRequest::SetName(name) => {
                self.set_name(name);
                sender.input(AddZoneDialogRequest::ValidateName);
            }
            AddZoneDialogRequest::SetDescription(desc) => {
                self.set_description(desc);
            }
            AddZoneDialogRequest::SetTarget(target) => {
                self.set_target(target);
            }
            AddZoneDialogRequest::ValidateName => {
                match validate_zone_name(&self.name) {
                    Ok(_) => {
                        self.set_name_valid(true);
                        self.set_name_error(None);
                    }
                    Err(e) => {
                        self.set_name_valid(false);
                        self.set_name_error(Some(e.user_message()));
                    }
                }
            }
            AddZoneDialogRequest::Add => {
                if self.name_valid && !self.name.is_empty() {
                    let settings = ZoneSettings {
                        name: self.name.clone(),
                        description: if self.description.is_empty() {
                            format!("Custom zone: {}", self.name)
                        } else {
                            self.description.clone()
                        },
                        target: self.target.clone(),
                        version: "1.0".to_string(),
                        ..Default::default()
                    };
                    
                    sender
                        .output(AddZoneDialogResponse::ZoneSettings(settings))
                        .unwrap();
                }
            }
            AddZoneDialogRequest::Cancel => {
                // Reset form
                self.set_name(String::new());
                self.set_description(String::new());
                self.set_target(ZoneTarget::Default);
                self.set_name_valid(false);
                self.set_name_error(None);
            }
        }
    }
}
