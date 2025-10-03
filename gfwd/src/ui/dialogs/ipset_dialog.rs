use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::core::validation::{validate_ipset_name, validate_ipset_type, validate_ipset_entry};
use crate::messages::ipset::{IPSetDialogRequest, IPSetDialogResponse};
use crate::models::IPSetSettings;

#[tracker::track]
#[derive(Debug)]
pub struct IPSetDialog {
    name: String,
    ipset_type: String,
    entries: Vec<String>,
    current_entry: String,
    name_valid: bool,
    name_error: Option<String>,
    type_valid: bool,
    type_error: Option<String>,
    entry_valid: bool,
    entry_error: Option<String>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for IPSetDialog {
    type Init = ();
    type Input = IPSetDialogRequest;
    type Output = IPSetDialogResponse;

    view! {
        dialog = adw::Dialog {
            set_title: "Create IP Set",
            set_content_width: 500,
            set_content_height: 700,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    add_css_class: "flat",

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Create IP Set",
                        set_subtitle: "Group IP addresses for firewall rules",
                    },

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        connect_clicked[sender, dialog] => move |_| {
                            sender.input(IPSetDialogRequest::Cancel);
                            dialog.close();
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Create",
                        add_css_class: "suggested-action",
                        #[track(model.changed(IPSetDialog::name_valid()) | model.changed(IPSetDialog::type_valid()))]
                        set_sensitive: model.name_valid && model.type_valid,
                        connect_clicked => IPSetDialogRequest::Create,
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,

                    adw::Clamp {
                        set_maximum_size: 400,

                        adw::PreferencesPage {
                            set_icon_name: Some("network-server-symbolic"),
                            set_title: "IP Set Configuration",
                            set_description: "Configure IP set properties and entries",

                            // Basic Configuration
                            add = &adw::PreferencesGroup {
                                set_title: "Basic Configuration",
                                set_description: Some("Set the name and type for the IP set"),

                                add = &adw::EntryRow {
                                    set_title: "Name",
                                    set_text: &model.name,
                                    connect_changed[sender] => move |entry| {
                                        sender.input(IPSetDialogRequest::SetName(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(IPSetDialogRequest::ValidateName);
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("folder-symbolic"),
                                    },
                                },

                                // Name validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(IPSetDialog::name_error()))]
                                    set_visible: model.name_error.is_some(),
                                    #[track(model.changed(IPSetDialog::name_error()))]
                                    set_title: &model.name_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },

                                add = &adw::ComboRow {
                                    set_title: "Type",
                                    set_subtitle: "Select the IP set type",
                                    set_model: Some(&gtk::StringList::new(&[
                                        "hash:ip",
                                        "hash:net", 
                                        "hash:ip,port",
                                        "hash:net,port",
                                        "hash:mac",
                                        "bitmap:ip",
                                        "list:set",
                                    ])),
                                    connect_selected_notify[sender] => move |combo| {
                                        if let Some(selected) = combo.selected_item() {
                                            if let Some(string_obj) = selected.downcast_ref::<gtk::StringObject>() {
                                                sender.input(IPSetDialogRequest::SetType(string_obj.string().to_string()));
                                            }
                                        }
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("preferences-system-symbolic"),
                                    },
                                },

                                // Type validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(IPSetDialog::type_error()))]
                                    set_visible: model.type_error.is_some(),
                                    #[track(model.changed(IPSetDialog::type_error()))]
                                    set_title: &model.type_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },
                            },

                            // Entry Management
                            add = &adw::PreferencesGroup {
                                set_title: "Entries",
                                set_description: Some("Add IP addresses, networks, or other entries based on the selected type"),

                                add = &adw::ActionRow {
                                    set_title: "Add Entry",
                                    set_subtitle: "Enter an IP address, network, or other entry",

                                    add_suffix = &gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 6,

                                        gtk::Entry {
                                            set_placeholder_text: Some("192.168.1.1 or 10.0.0.0/8"),
                                            set_text: &model.current_entry,
                                            set_hexpand: true,
                                            connect_changed[sender] => move |entry| {
                                                sender.input(IPSetDialogRequest::SetCurrentEntry(entry.text().to_string()));
                                            },
                                            connect_activate[sender] => move |_| {
                                                sender.input(IPSetDialogRequest::AddEntry);
                                            },
                                        },

                                        gtk::Button {
                                            set_icon_name: "list-add-symbolic",
                                            set_tooltip_text: Some("Add entry"),
                                            add_css_class: "flat",
                                            #[track(model.changed(IPSetDialog::entry_valid()) | model.changed(IPSetDialog::current_entry()))]
                                            set_sensitive: model.entry_valid && !model.current_entry.is_empty(),
                                            connect_clicked => IPSetDialogRequest::AddEntry,
                                        },
                                    },
                                },

                                // Entry validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(IPSetDialog::entry_error()))]
                                    set_visible: model.entry_error.is_some(),
                                    #[track(model.changed(IPSetDialog::entry_error()))]
                                    set_title: &model.entry_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },

                                // Current entries display
                                add = &adw::ActionRow {
                                    #[track(model.changed(IPSetDialog::entries()))]
                                    set_visible: !model.entries.is_empty(),
                                    #[track(model.changed(IPSetDialog::entries()))]
                                    set_title: &format!("Entries ({})", model.entries.len()),
                                    #[track(model.changed(IPSetDialog::entries()))]
                                    set_subtitle: &model.entries.join(", "),

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("emblem-ok-symbolic"),
                                        add_css_class: "success",
                                    },
                                },
                            },

                            // Information
                            add = &adw::PreferencesGroup {
                                set_title: "Information",
                                set_description: Some("IP set type descriptions and usage examples"),

                                add = &adw::ActionRow {
                                    set_title: "Type Examples",
                                    set_subtitle: "hash:ip - Single IP addresses\nhash:net - Network ranges with CIDR\nhash:ip,port - IP and port combinations\nhash:mac - MAC addresses",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-information-symbolic"),
                                        add_css_class: "accent",
                                    },
                                },
                            },
                        }
                    }
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            name: String::new(),
            ipset_type: "hash:ip".to_string(), // Default type
            entries: Vec::new(),
            current_entry: String::new(),
            name_valid: false,
            name_error: None,
            type_valid: true, // Default type is valid
            type_error: None,
            entry_valid: false,
            entry_error: None,
            tracker: 0,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: IPSetDialogRequest, sender: AsyncComponentSender<Self>) {
        self.reset();
        match msg {
            IPSetDialogRequest::SetName(name) => {
                self.set_name(name);
                sender.input(IPSetDialogRequest::ValidateName);
            }
            IPSetDialogRequest::ValidateName => {
                match validate_ipset_name(&self.name) {
                    Ok(_) => {
                        self.set_name_valid(true);
                        self.set_name_error(None);
                    }
                    Err(e) => {
                        self.set_name_valid(false);
                        self.set_name_error(Some(e.user_message().to_string()));
                    }
                }
            }
            IPSetDialogRequest::SetType(ipset_type) => {
                self.set_ipset_type(ipset_type);
                sender.input(IPSetDialogRequest::ValidateType);
            }
            IPSetDialogRequest::ValidateType => {
                match validate_ipset_type(&self.ipset_type) {
                    Ok(_) => {
                        self.set_type_valid(true);
                        self.set_type_error(None);
                    }
                    Err(e) => {
                        self.set_type_valid(false);
                        self.set_type_error(Some(e.user_message().to_string()));
                    }
                }
            }
            IPSetDialogRequest::SetCurrentEntry(entry) => {
                self.set_current_entry(entry);
                sender.input(IPSetDialogRequest::ValidateCurrentEntry);
            }
            IPSetDialogRequest::ValidateCurrentEntry => {
                if self.current_entry.trim().is_empty() {
                    self.set_entry_valid(false);
                    self.set_entry_error(None);
                } else {
                    match validate_ipset_entry(&self.current_entry, &self.ipset_type) {
                        Ok(_) => {
                            self.set_entry_valid(true);
                            self.set_entry_error(None);
                        }
                        Err(e) => {
                            self.set_entry_valid(false);
                            self.set_entry_error(Some(e.user_message().to_string()));
                        }
                    }
                }
            }
            IPSetDialogRequest::AddEntry => {
                if self.entry_valid && !self.current_entry.trim().is_empty() {
                    let entry = self.current_entry.trim().to_string();
                    if !self.entries.contains(&entry) {
                        let mut new_entries = self.entries.clone();
                        new_entries.push(entry);
                        self.set_entries(new_entries);
                        self.set_current_entry(String::new());
                        self.set_entry_valid(false);
                        self.set_entry_error(None);
                    }
                }
            }
            IPSetDialogRequest::RemoveEntry(entry) => {
                let mut new_entries = self.entries.clone();
                new_entries.retain(|e| e != &entry);
                self.set_entries(new_entries);
            }
            IPSetDialogRequest::Create => {
                if self.name_valid && self.type_valid {
                    let settings = IPSetSettings {
                        name: self.name.clone(),
                        ipset_type: self.ipset_type.clone(),
                        entries: self.entries.clone(),
                        options: std::collections::HashMap::new(),
                    };
                    let _ = sender.output(IPSetDialogResponse::IPSetCreated { settings });
                    sender.input(IPSetDialogRequest::Cancel);
                }
            }
            IPSetDialogRequest::Cancel => {
                // Reset form
                self.set_name(String::new());
                self.set_ipset_type("hash:ip".to_string());
                self.set_entries(Vec::new());
                self.set_current_entry(String::new());
                self.set_name_valid(false);
                self.set_name_error(None);
                self.set_type_valid(true);
                self.set_type_error(None);
                self.set_entry_valid(false);
                self.set_entry_error(None);
            }
        }
    }
}