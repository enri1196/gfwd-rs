use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::messages::icmp::{IcmpDialogRequest, IcmpDialogResponse};
use crate::models::IcmpType;

#[tracker::track]
#[derive(Debug)]
pub struct AddIcmpDialog {
    available_icmp_types: Vec<IcmpType>,
    selected_icmp: Option<String>,
    selected_index: u32,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AddIcmpDialog {
    type Init = Vec<IcmpType>;
    type Input = IcmpDialogRequest;
    type Output = IcmpDialogResponse;

    view! {
        dialog = adw::Dialog {
            set_title: "Add ICMP Block",
            set_content_width: 400,
            set_content_height: 500,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    add_css_class: "flat",

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Add ICMP Block",
                        set_subtitle: "Block ICMP messages for this zone",
                    },

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        set_accessible_role: gtk::AccessibleRole::Button,
                        set_can_focus: true,
                        connect_clicked[sender, root] => move |_| {
                            sender.input(IcmpDialogRequest::Cancel);
                            root.close();
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Add Block",
                        add_css_class: "suggested-action",
                        set_accessible_role: gtk::AccessibleRole::Button,
                        set_can_focus: true,
                        set_receives_default: true,
                        #[track(model.changed(AddIcmpDialog::selected_icmp()))]
                        set_sensitive: model.selected_icmp.is_some(),
                        connect_clicked[sender, root] => move |_| {
                            sender.input(IcmpDialogRequest::Add);
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
                            set_title: "ICMP Configuration",
                            set_description: "Select which ICMP message type to block in this zone",

                            add = &adw::PreferencesGroup {
                                set_title: "ICMP Type Selection",
                                set_description: Some("Choose an ICMP message type to block"),

                                // ICMP type selection
                                add = &adw::ComboRow {
                                    set_title: "ICMP Message Type",
                                    set_subtitle: "Type of ICMP message to block",
                                    set_accessible_role: gtk::AccessibleRole::ComboBox,
                                    set_can_focus: true,
                                    #[track(model.changed(AddIcmpDialog::available_icmp_types()))]
                                    set_model: Some(&{
                                        let string_list = gtk::StringList::new(&[]);
                                        for icmp_type in &model.available_icmp_types {
                                            string_list.append(&icmp_type.name);
                                        }
                                        string_list
                                    }),
                                    #[track(model.changed(AddIcmpDialog::selected_index()))]
                                    set_selected: model.selected_index,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-wired-symbolic"),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                    },

                                    connect_selected_notify[sender, available_types = model.available_icmp_types.clone()] => move |combo| {
                                        let selected_idx = combo.selected() as usize;
                                        if let Some(icmp_type) = available_types.get(selected_idx) {
                                            sender.input(IcmpDialogRequest::SetSelectedIcmp(icmp_type.name.clone()));
                                        }
                                    },
                                },

                                // Selected ICMP type description
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddIcmpDialog::selected_icmp()))]
                                    set_visible: model.selected_icmp.is_some(),
                                    set_title: "Description",
                                    #[track(model.changed(AddIcmpDialog::selected_icmp()) | model.changed(AddIcmpDialog::available_icmp_types()))]
                                    set_subtitle: &{
                                        if let Some(ref selected) = model.selected_icmp {
                                            model.available_icmp_types
                                                .iter()
                                                .find(|t| t.name == *selected)
                                                .map(|t| t.description.as_str())
                                                .unwrap_or("No description available")
                                        } else {
                                            ""
                                        }
                                    },
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-information-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },
                            },

                            add = &adw::PreferencesGroup {
                                set_title: "Block Configuration",
                                set_description: Some("Information about ICMP blocking"),

                                // Information about ICMP blocking
                                add = &adw::ActionRow {
                                    set_title: "Block Effect",
                                    set_subtitle: "Selected ICMP messages will be dropped for this zone",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("security-high-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "warning",
                                    },
                                },

                                // Zone scope information
                                add = &adw::ActionRow {
                                    set_title: "Scope",
                                    set_subtitle: "This block will only apply to the current zone",
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("folder-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "accent",
                                    },
                                },

                                // Summary of action
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddIcmpDialog::selected_icmp()))]
                                    set_visible: model.selected_icmp.is_some(),
                                    set_title: "Action Summary",
                                    #[track(model.changed(AddIcmpDialog::selected_icmp()))]
                                    set_subtitle: &{
                                        if let Some(ref selected) = model.selected_icmp {
                                            format!("Block '{}' ICMP messages", selected)
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
        available_icmp_types: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AddIcmpDialog {
            available_icmp_types,
            selected_icmp: None,
            selected_index: 0,
            tracker: 0,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();

        match msg {
            IcmpDialogRequest::SetSelectedIcmp(name) => {
                // Find the index of the selected ICMP type
                if let Some(index) = self
                    .available_icmp_types
                    .iter()
                    .position(|t| t.name == name)
                {
                    self.set_selected_index(index as u32);
                    self.set_selected_icmp(Some(name));
                }
            }
            IcmpDialogRequest::Add => {
                if let Some(ref selected) = self.selected_icmp {
                    sender
                        .output(IcmpDialogResponse::IcmpSelected {
                            name: selected.clone(),
                        })
                        .unwrap();
                }
            }
            IcmpDialogRequest::Cancel => {
                // Reset form
                self.set_selected_icmp(None);
                self.set_selected_index(0);
            }
        }
    }
}
