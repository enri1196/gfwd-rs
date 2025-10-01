use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::core::validation::validate_source_address;
use crate::messages::source::{SourceDialogRequest, SourceDialogResponse};

#[tracker::track]
#[derive(Debug)]
pub struct AddSourceDialog {
    source_address: String,
    source_error: Option<String>,
    is_valid: bool,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AddSourceDialog {
    type Init = ();
    type Input = SourceDialogRequest;
    type Output = SourceDialogResponse;

    view! {
        dialog = adw::Dialog {
            set_title: "Add Source",
            set_content_width: 400,
            set_content_height: 500,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    add_css_class: "flat",

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Add Source",
                        set_subtitle: "Add source address or network to zone",
                    },

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        connect_clicked[sender, root] => move |_| {
                            sender.input(SourceDialogRequest::Cancel);
                            root.close();
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Add Source",
                        add_css_class: "suggested-action",
                        #[track(model.changed(AddSourceDialog::is_valid()))]
                        set_sensitive: model.is_valid,
                        connect_clicked[sender, root] => move |_| {
                            sender.input(SourceDialogRequest::Add);
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
                            set_icon_name: Some("network-server-symbolic"),
                            set_title: "Source Configuration",
                            set_description: "Add a source IP address or network range to this zone",

                            add = &adw::PreferencesGroup {
                                set_title: "Source Details",
                                set_description: Some("Enter an IP address or network range"),

                                // Source address input
                                add = &adw::EntryRow {
                                    set_title: "Source Address",
                                    set_text: &model.source_address,
                                    #[track(model.changed(AddSourceDialog::source_error()))]
                                    add_css_class: if model.source_error.is_some() { "error" } else { "" },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-server-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    connect_changed[sender] => move |entry| {
                                        let text = entry.text().to_string();
                                        sender.input(SourceDialogRequest::SetSource(text));
                                    },

                                    connect_apply[sender] => move |_| {
                                        sender.input(SourceDialogRequest::ValidateSource);
                                    },
                                },

                                // Source validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddSourceDialog::source_error()))]
                                    set_visible: model.source_error.is_some(),
                                    #[track(model.changed(AddSourceDialog::source_error()))]
                                    set_title: model.source_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "error",
                                    },
                                },
                            },

                            add = &adw::PreferencesGroup {
                                set_title: "Address Format Examples",
                                set_description: Some("Supported IP address and network formats"),

                                // IPv4 examples
                                add = &adw::ActionRow {
                                    set_title: "IPv4 Address",
                                    set_subtitle: "192.168.1.100, 10.0.0.1",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-server-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "IPv4 Network",
                                    set_subtitle: "192.168.1.0/24, 10.0.0.0/8",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-wired-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "IPv6 Address",
                                    set_subtitle: "::1, 2001:db8::1, fe80::1",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-server-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },
                            },

                            add = &adw::PreferencesGroup {
                                set_title: "Zone Assignment",
                                set_description: Some("Source address assignment information"),

                                // Zone scope information
                                add = &adw::ActionRow {
                                    set_title: "Assignment Effect",
                                    set_subtitle: "Traffic from this source will be processed by this zone",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("security-medium-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "success",
                                    },
                                },

                                // Priority information
                                add = &adw::ActionRow {
                                    set_title: "Priority",
                                    set_subtitle: "Source-based rules take precedence over interface-based rules",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-information-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },

                                // Summary of action
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddSourceDialog::source_address()) | model.changed(AddSourceDialog::is_valid()))]
                                    set_visible: model.is_valid && !model.source_address.is_empty(),
                                    set_title: "Action Summary",
                                    #[track(model.changed(AddSourceDialog::source_address()))]
                                    set_subtitle: &{
                                        if !model.source_address.is_empty() {
                                            format!("Add '{}' source to this zone", model.source_address)
                                        } else {
                                            String::new()
                                        }
                                    },
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("emblem-ok-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "success",
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
        let model = AddSourceDialog {
            source_address: String::new(),
            source_error: None,
            is_valid: false,
            tracker: 0,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();

        match msg {
            SourceDialogRequest::SetSource(address) => {
                self.set_source_address(address);
                // Auto-validate on change
                sender.input(SourceDialogRequest::ValidateSource);
            }
            SourceDialogRequest::ValidateSource => {
                match validate_source_address(&self.source_address) {
                    Ok(_) => {
                        self.set_source_error(None);
                        self.set_is_valid(!self.source_address.trim().is_empty());
                    }
                    Err(err) => {
                        self.set_source_error(Some(err.to_string()));
                        self.set_is_valid(false);
                    }
                }
            }
            SourceDialogRequest::Add => {
                if self.is_valid && !self.source_address.trim().is_empty() {
                    sender
                        .output(SourceDialogResponse::SourceAdded {
                            address: self.source_address.trim().to_string(),
                        })
                        .unwrap();
                }
            }
            SourceDialogRequest::Cancel => {
                // Reset form
                self.set_source_address(String::new());
                self.set_source_error(None);
                self.set_is_valid(false);
            }
        }
    }
}