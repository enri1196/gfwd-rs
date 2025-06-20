use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::fwd_broker::ZoneSettings;

#[derive(Debug)]
pub struct AddZoneDialog {
    name: String,
    description: String,
}

#[derive(Debug)]
pub enum AddZoneDialogRequest {
    SetName(String),
    SetDescription(String),
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
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                set_margin_all: 12,

                // Name field
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,

                    append = &gtk::Label {
                        set_text: "Zone Name:",
                        set_halign: gtk::Align::Start,
                    },

                    append = &gtk::Entry {
                        set_hexpand: true,
                        set_placeholder_text: Some("Enter zone name"),
                        connect_changed[sender] => move |entry| {
                            sender.input(AddZoneDialogRequest::SetName(entry.text().to_string()));
                        }
                    },
                },

                // Description field
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,

                    append = &gtk::Label {
                        set_text: "Description:",
                        set_halign: gtk::Align::Start,
                    },

                    append = &gtk::Entry {
                        set_hexpand: true,
                        set_placeholder_text: Some("Enter description"),
                        connect_changed[sender] => move |entry| {
                            sender.input(AddZoneDialogRequest::SetDescription(entry.text().to_string()));
                        }
                    },
                },

                // Action buttons
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::End,
                    set_spacing: 6,
                    set_margin_top: 12,

                    append = &gtk::Button::with_label("Cancel") {
                        connect_clicked[sender, root] => move |_| {
                            sender.input(AddZoneDialogRequest::Cancel);
                            root.close();
                        },
                    },

                    append = &gtk::Button::with_label("Add") {
                        add_css_class: "suggested-action",
                        connect_clicked[sender, root] => move |_| {
                            sender.input(AddZoneDialogRequest::Add);
                            root.close();
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
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            AddZoneDialogRequest::SetName(name) => {
                self.name = name;
            }
            AddZoneDialogRequest::SetDescription(desc) => {
                self.description = desc;
            }
            AddZoneDialogRequest::Add => {
                if !self.name.is_empty() && !self.description.is_empty() {
                    sender
                        .output(AddZoneDialogResponse::ZoneSettings(ZoneSettings {
                            name: self.name.clone(),
                            description: self.description.clone(),
                            ..Default::default()
                        }))
                        .unwrap();
                }
            }
            AddZoneDialogRequest::Cancel => {}
        }
    }
}
