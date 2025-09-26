use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::core::validation::validate_zone_name;
use crate::messages::zone::{ZoneDialogRequest, ZoneDialogResponse};
use crate::models::{ZoneSettings, ZoneTarget};

#[tracker::track]
#[derive(Debug)]
pub struct AddZoneDialog {
    name: String,
    description: String,
    target: ZoneTarget,
    name_valid: bool,
    name_error: Option<String>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AddZoneDialog {
    type Init = ();
    type Input = ZoneDialogRequest;
    type Output = ZoneDialogResponse;

    view! {
        dialog = adw::Dialog {
            set_title: "Add Firewall Zone",
            set_content_width: 400,
            set_content_height: 500,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    add_css_class: "flat",

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Add Zone",
                        set_subtitle: "Create a new firewall zone",
                    },

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        connect_clicked[sender, root] => move |_| {
                            sender.input(ZoneDialogRequest::Cancel);
                            root.close();
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Add Zone",
                        add_css_class: "suggested-action",
                        #[track(model.changed(AddZoneDialog::name_valid()))]
                        set_sensitive: model.name_valid && !model.name.is_empty(),
                        connect_clicked[sender, root] => move |_| {
                            sender.input(ZoneDialogRequest::Add);
                            root.close();
                        },
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                    set_vexpand: true,

                    #[wrap(Some)]
                    set_child = &adw::Clamp {
                        set_maximum_size: 400,
                        set_tightening_threshold: 350,

                        adw::PreferencesPage {
                            set_icon_name: Some("security-high-symbolic"),
                            set_title: "Zone Configuration",
                            set_description: "Configure the properties of your new firewall zone",

                            add = &adw::PreferencesGroup {
                                set_title: "Basic Information",
                                set_description: Some("Set the name and description for this zone"),

                                // Zone name with icon
                                add = &adw::EntryRow {
                                    set_title: "Zone Name",
                                    #[watch]
                                    set_text: &model.name,
                                    #[track(model.changed(AddZoneDialog::name_error()))]
                                    set_css_classes: if model.name_error.is_some() { &["error"] } else { &[] },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("folder-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    add_suffix = &gtk::Label {
                                        set_text: "Required",
                                        add_css_class: "dim-label",
                                        add_css_class: "caption",
                                    },

                                    connect_changed[sender] => move |entry| {
                                        sender.input(ZoneDialogRequest::SetName(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(ZoneDialogRequest::ValidateName);
                                    },
                                },

                                // Name validation feedback
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddZoneDialog::name_error()))]
                                    set_visible: model.name_error.is_some(),
                                    #[track(model.changed(AddZoneDialog::name_error()))]
                                    set_title: model.name_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "error",
                                    },
                                },

                                // Description field
                                add = &adw::EntryRow {
                                    set_title: "Description",
                                    #[watch]
                                    set_text: &model.description,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("text-x-generic-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    add_suffix = &gtk::Label {
                                        set_text: "Optional",
                                        add_css_class: "dim-label",
                                        add_css_class: "caption",
                                    },

                                    connect_changed[sender] => move |entry| {
                                        sender.input(ZoneDialogRequest::SetDescription(entry.text().to_string()));
                                    },
                                },
                            },

                            add = &adw::PreferencesGroup {
                                set_title: "Security Policy",
                                set_description: Some("Configure the default behavior for this zone"),

                                // Target selection with better styling
                                add = &adw::ComboRow {
                                    set_title: "Default Target",
                                    set_subtitle: "Action for packets not matching any rule",
                                    set_model: Some(&gtk::StringList::new(&[
                                        "Default",
                                        "Accept",
                                        "Drop",
                                        "Reject"
                                    ])),
                                    set_selected: match model.target {
                                        ZoneTarget::Default => 0,
                                        ZoneTarget::Accept => 1,
                                        ZoneTarget::Drop => 2,
                                        ZoneTarget::Reject => 3,
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("security-medium-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    connect_selected_notify[sender] => move |combo| {
                                        let target = match combo.selected() {
                                            0 => ZoneTarget::Default,
                                            1 => ZoneTarget::Accept,
                                            2 => ZoneTarget::Drop,
                                            3 => ZoneTarget::Reject,
                                            _ => ZoneTarget::Default,
                                        };
                                        sender.input(ZoneDialogRequest::SetTarget(target));
                                    },
                                },

                                // Target explanation
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddZoneDialog::target()))]
                                    set_title: match model.target {
                                        ZoneTarget::Default => "Uses system default policy",
                                        ZoneTarget::Accept => "Allows all unmatched traffic",
                                        ZoneTarget::Drop => "Silently drops unmatched packets",
                                        ZoneTarget::Reject => "Rejects with ICMP response",
                                    },
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        #[track(model.changed(AddZoneDialog::target()))]
                                        set_icon_name: Some(match model.target {
                                            ZoneTarget::Default => "preferences-system-symbolic",
                                            ZoneTarget::Accept => "emblem-ok-symbolic",
                                            ZoneTarget::Drop => "action-unavailable-symbolic",
                                            ZoneTarget::Reject => "dialog-error-symbolic",
                                        }),
                                        set_pixel_size: 16,
                                        #[track(model.changed(AddZoneDialog::target()))]
                                        add_css_class: match model.target {
                                            ZoneTarget::Accept => "success",
                                            ZoneTarget::Drop => "warning",
                                            ZoneTarget::Reject => "error",
                                            _ => "",
                                        },
                                    },
                                },
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
            ZoneDialogRequest::SetName(name) => {
                self.set_name(name);
                sender.input(ZoneDialogRequest::ValidateName);
            }
            ZoneDialogRequest::SetDescription(desc) => {
                self.set_description(desc);
            }
            ZoneDialogRequest::SetTarget(target) => {
                self.set_target(target);
            }
            ZoneDialogRequest::ValidateName => match validate_zone_name(&self.name) {
                Ok(_) => {
                    self.set_name_valid(true);
                    self.set_name_error(None);
                }
                Err(e) => {
                    self.set_name_valid(false);
                    self.set_name_error(Some(e.user_message()));
                }
            },
            ZoneDialogRequest::Add => {
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
                        .output(ZoneDialogResponse::ZoneSettings(settings))
                        .unwrap();
                }
            }
            ZoneDialogRequest::Cancel => {
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
