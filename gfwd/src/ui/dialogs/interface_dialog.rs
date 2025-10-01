use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::core::validation::validate_interface_name;
use crate::messages::interface::{InterfaceDialogRequest, InterfaceDialogResponse};

#[tracker::track]
#[derive(Debug)]
pub struct AddInterfaceDialog {
    interface_name: String,
    interface_error: Option<String>,
    is_valid: bool,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AddInterfaceDialog {
    type Init = ();
    type Input = InterfaceDialogRequest;
    type Output = InterfaceDialogResponse;

    view! {
        dialog = adw::Dialog {
            set_title: "Add Interface",
            set_content_width: 400,
            set_content_height: 500,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    add_css_class: "flat",

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Add Interface",
                        set_subtitle: "Assign network interface to zone",
                    },

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        connect_clicked[sender, root] => move |_| {
                            sender.input(InterfaceDialogRequest::Cancel);
                            root.close();
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Add Interface",
                        add_css_class: "suggested-action",
                        #[track(model.changed(AddInterfaceDialog::is_valid()))]
                        set_sensitive: model.is_valid,
                        connect_clicked[sender, root] => move |_| {
                            sender.input(InterfaceDialogRequest::Add);
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
                            set_icon_name: Some("network-wired-symbolic"),
                            set_title: "Interface Configuration",
                            set_description: "Assign a network interface to this firewall zone",

                            add = &adw::PreferencesGroup {
                                set_title: "Interface Details",
                                set_description: Some("Enter the name of the network interface"),

                                // Interface name input
                                add = &adw::EntryRow {
                                    set_title: "Interface Name",
                                    set_text: &model.interface_name,
                                    #[track(model.changed(AddInterfaceDialog::interface_error()))]
                                    add_css_class: if model.interface_error.is_some() { "error" } else { "" },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-wired-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    connect_changed[sender] => move |entry| {
                                        let text = entry.text().to_string();
                                        sender.input(InterfaceDialogRequest::SetInterface(text));
                                    },

                                    connect_apply[sender] => move |_| {
                                        sender.input(InterfaceDialogRequest::ValidateInterface);
                                    },
                                },

                                // Interface validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddInterfaceDialog::interface_error()))]
                                    set_visible: model.interface_error.is_some(),
                                    #[track(model.changed(AddInterfaceDialog::interface_error()))]
                                    set_title: model.interface_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "error",
                                    },
                                },
                            },

                            add = &adw::PreferencesGroup {
                                set_title: "Interface Information",
                                set_description: Some("Common network interface examples"),

                                // Examples
                                add = &adw::ActionRow {
                                    set_title: "Ethernet Interfaces",
                                    set_subtitle: "eth0, enp0s3, ens33",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-wired-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Wireless Interfaces",
                                    set_subtitle: "wlan0, wlp2s0, wifi0",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-wireless-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Virtual Interfaces",
                                    set_subtitle: "docker0, br0, veth0",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("folder-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },
                            },

                            add = &adw::PreferencesGroup {
                                set_title: "Zone Assignment",
                                set_description: Some("Interface assignment information"),

                                // Zone scope information
                                add = &adw::ActionRow {
                                    set_title: "Assignment Effect",
                                    set_subtitle: "Interface traffic will be processed by this zone's rules",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("security-medium-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "success",
                                    },
                                },

                                // Summary of action
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddInterfaceDialog::interface_name()) | model.changed(AddInterfaceDialog::is_valid()))]
                                    set_visible: model.is_valid && !model.interface_name.is_empty(),
                                    set_title: "Action Summary",
                                    #[track(model.changed(AddInterfaceDialog::interface_name()))]
                                    set_subtitle: &{
                                        if !model.interface_name.is_empty() {
                                            format!("Assign '{}' interface to this zone", model.interface_name)
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
        let model = AddInterfaceDialog {
            interface_name: String::new(),
            interface_error: None,
            is_valid: false,
            tracker: 0,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();

        match msg {
            InterfaceDialogRequest::SetInterface(name) => {
                self.set_interface_name(name);
                // Auto-validate on change
                sender.input(InterfaceDialogRequest::ValidateInterface);
            }
            InterfaceDialogRequest::ValidateInterface => {
                match validate_interface_name(&self.interface_name) {
                    Ok(_) => {
                        self.set_interface_error(None);
                        self.set_is_valid(!self.interface_name.trim().is_empty());
                    }
                    Err(err) => {
                        self.set_interface_error(Some(err.to_string()));
                        self.set_is_valid(false);
                    }
                }
            }
            InterfaceDialogRequest::Add => {
                if self.is_valid && !self.interface_name.trim().is_empty() {
                    sender
                        .output(InterfaceDialogResponse::InterfaceAdded {
                            name: self.interface_name.trim().to_string(),
                        })
                        .unwrap();
                }
            }
            InterfaceDialogRequest::Cancel => {
                // Reset form
                self.set_interface_name(String::new());
                self.set_interface_error(None);
                self.set_is_valid(false);
            }
        }
    }
}