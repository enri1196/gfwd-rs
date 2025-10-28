use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::core::FwdBroker;
use crate::core::validation::validate_ipset_entry;
use crate::messages::ipset::{IPSetViewRequest, IPSetViewResponse};
use crate::models::IPSetSettings;

use crate::ui::components::{IPSetEntryItem, IPSetEntryItemResponse, IPSetItem, IPSetItemResponse};
use crate::ui::dialogs::IPSetDialog;

#[tracker::track]
#[derive(Debug)]
pub struct IPSetView {
    #[tracker::do_not_track]
    broker: &'static FwdBroker,
    #[tracker::do_not_track]
    ipset_dialog: AsyncController<IPSetDialog>,
    #[tracker::do_not_track]
    ipsets: FactoryVecDeque<IPSetItem>,
    #[tracker::do_not_track]
    ipset_entries: FactoryVecDeque<IPSetEntryItem>,
    ipset_list: Vec<String>,
    selected_ipset: Option<String>,
    ipset_details: Option<IPSetSettings>,
    entry_input: String,
    entry_valid: bool,
    entry_error: Option<String>,
    details_loading: bool,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for IPSetView {
    type Init = &'static FwdBroker;
    type Input = IPSetViewRequest;
    type Output = IPSetViewResponse;

    view! {
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_vscrollbar_policy: gtk::PolicyType::Automatic,

            adw::Clamp {
                set_maximum_size: 900,
                set_tightening_threshold: 600,

                adw::PreferencesPage {
                    set_icon_name: Some("network-server-symbolic"),
                    set_title: "IP Sets",
                    set_description: "Manage IP sets for efficient address grouping",

                    add = &adw::PreferencesGroup {
                        set_title: "IP Set Management",
                        set_description: Some("Create and manage IP sets to group IP addresses for firewall rules"),

                        #[wrap(Some)]
                        set_header_suffix = &gtk::Button {
                            set_icon_name: "list-add-symbolic",
                            set_tooltip_text: Some("Create new IP set"),
                            set_accessible_role: gtk::AccessibleRole::Button,
                            set_can_focus: true,
                            add_css_class: "flat",
                            connect_clicked => IPSetViewRequest::ShowCreateDialog,
                        },

                        #[local_ref]
                        ipset_list_box -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_accessible_role: gtk::AccessibleRole::List,
                            set_can_focus: true,
                            #[track(model.changed(IPSetView::ipset_list()))]
                            set_visible: !model.ipset_list.is_empty(),
                        },

                        add = &adw::ActionRow {
                            #[track(model.changed(IPSetView::ipset_list()))]
                            set_visible: model.ipset_list.is_empty(),
                            set_title: "No IP sets configured",
                            set_subtitle: "Create an IP set to group IP addresses for firewall rules",

                            add_prefix = &gtk::Image {
                                set_icon_name: Some("network-server-symbolic"),
                                add_css_class: "dim-label",
                            },
                        },
                    },

                    add = &adw::PreferencesGroup {
                        set_title: "Selected IP Set",
                        set_description: Some("View details and manage entries"),
                        #[track(model.changed(IPSetView::selected_ipset()) | model.changed(IPSetView::details_loading()))]
                        set_visible: model.selected_ipset.is_some() || model.details_loading,

                        add = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            set_valign: gtk::Align::Center,
                            #[track(model.changed(IPSetView::details_loading()))]
                            set_visible: model.details_loading,

                            gtk::Spinner {
                                set_spinning: true,
                            },

                            gtk::Label {
                                set_text: "Loading IP set details...",
                                add_css_class: "dim-label",
                            },
                        },

                        add = &adw::ActionRow {
                            #[track(model.changed(IPSetView::selected_ipset()) | model.changed(IPSetView::details_loading()))]
                            set_visible: model.selected_ipset.is_some() && !model.details_loading,
                            set_title: "Name",
                            #[track(model.changed(IPSetView::selected_ipset()))]
                            set_subtitle: model.selected_ipset.as_deref().unwrap_or(""),
                            add_prefix = &gtk::Image {
                                set_icon_name: Some("selection-mode-symbolic"),
                                add_css_class: "accent",
                            },
                        },

                        add = &adw::ActionRow {
                            #[track(model.changed(IPSetView::ipset_details()) | model.changed(IPSetView::details_loading()))]
                            set_visible: model.ipset_details.is_some() && !model.details_loading,
                            set_title: "Type",
                            #[track(model.changed(IPSetView::ipset_details()))]
                            set_subtitle: model.selected_ipset_type(),
                            add_prefix = &gtk::Image {
                                set_icon_name: Some("preferences-system-symbolic"),
                                add_css_class: "accent",
                            },
                        },

                        add = &adw::ActionRow {
                            #[track(model.changed(IPSetView::ipset_details()) | model.changed(IPSetView::details_loading()))]
                            set_visible: model.ipset_details.is_some() && !model.details_loading,
                            set_title: "Entries",
                            set_subtitle: &format!("{} total entries", model.selected_ipset_entry_count()),
                            add_prefix = &gtk::Image {
                                set_icon_name: Some("list-bullet-symbolic"),
                                add_css_class: "accent",
                            },
                        },

                        add = &adw::ActionRow {
                            #[track(model.changed(IPSetView::ipset_details()) | model.changed(IPSetView::details_loading()))]
                            set_visible: model.ipset_details.is_some() && !model.details_loading,
                            set_title: "Options",
                            set_subtitle: &format!("{} configured options", model.selected_ipset_option_count()),
                            add_prefix = &gtk::Image {
                                set_icon_name: Some("emblem-system-symbolic"),
                                add_css_class: "accent",
                            },
                        },
                    },

                    add = &adw::PreferencesGroup {
                        set_title: "Entries",
                        set_description: Some("Add or remove entries for the selected IP set"),
                        #[track(model.changed(IPSetView::ipset_details()))]
                        set_visible: model.ipset_details.is_some(),

                        add = &adw::ActionRow {
                            set_title: "Add Entry",
                            set_subtitle: "Entries must match the selected IP set type",

                            add_suffix = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 6,

                                gtk::Entry {
                                    #[track(model.changed(IPSetView::entry_input()))]
                                    set_text: &model.entry_input,
                                    set_placeholder_text: Some("192.168.1.1 or 10.0.0.0/8"),
                                    set_hexpand: true,
                                    connect_changed[sender] => move |entry| {
                                        sender.input(IPSetViewRequest::UpdateEntryInput(entry.text().to_string()));
                                    },
                                    connect_activate => IPSetViewRequest::AddEntry,
                                },

                                gtk::Button {
                                    set_icon_name: "list-add-symbolic",
                                    set_tooltip_text: Some("Add entry"),
                                    add_css_class: "flat",
                                    #[track(model.changed(IPSetView::entry_valid()) | model.changed(IPSetView::entry_input()) | model.changed(IPSetView::selected_ipset()) | model.changed(IPSetView::details_loading()))]
                                    set_sensitive: model.entry_valid && !model.entry_input.is_empty() && model.selected_ipset.is_some() && !model.details_loading,
                                    connect_clicked => IPSetViewRequest::AddEntry,
                                },
                            },
                        },

                        add = &adw::ActionRow {
                            #[track(model.changed(IPSetView::entry_error()))]
                            set_visible: model.entry_error.is_some(),
                            #[track(model.changed(IPSetView::entry_error()))]
                            set_title: &model.entry_error.as_deref().unwrap_or(""),
                            add_css_class: "error",

                            add_prefix = &gtk::Image {
                                set_icon_name: Some("dialog-warning-symbolic"),
                                add_css_class: "error",
                            },
                        },

                        #[local_ref]
                        ipset_entries_list_box -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_accessible_role: gtk::AccessibleRole::List,
                            #[track(model.changed(IPSetView::ipset_details()))]
                            set_visible: model
                                .ipset_details
                                .as_ref()
                                .map(|details| !details.entries.is_empty())
                                .unwrap_or(false),
                        },

                        add = &adw::ActionRow {
                            #[track(model.changed(IPSetView::ipset_details()))]
                            set_visible: model
                                .ipset_details
                                .as_ref()
                                .map(|details| details.entries.is_empty())
                                .unwrap_or(true),
                            set_title: "No entries configured",
                            set_subtitle: "Add entries to populate this IP set",

                            add_prefix = &gtk::Image {
                                set_icon_name: Some("list-add-symbolic"),
                                add_css_class: "dim-label",
                            },
                        },
                    },

                    add = &adw::PreferencesGroup {
                        set_title: "Selection",
                        #[track(model.changed(IPSetView::selected_ipset()) | model.changed(IPSetView::details_loading()) | model.changed(IPSetView::ipset_list()))]
                        set_visible: model.selected_ipset.is_none() && !model.details_loading && !model.ipset_list.is_empty(),

                        add = &adw::ActionRow {
                            set_title: "Select an IP set to manage entries",
                            set_subtitle: "Choose an IP set from the list to view its details",

                            add_prefix = &gtk::Image {
                                set_icon_name: Some("selection-mode-symbolic"),
                                add_css_class: "dim-label",
                            },
                        },
                    },
                }
            }
        }
    }

    async fn init(
        broker: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let ipset_dialog =
            IPSetDialog::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    crate::messages::ipset::IPSetDialogResponse::IPSetCreated { settings } => {
                        IPSetViewRequest::CreateIPSet(settings)
                    }
                });

        let ipsets = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |msg| match msg {
                IPSetItemResponse::Delete(name) => IPSetViewRequest::DeleteIPSet(name),
                IPSetItemResponse::Select(name) => IPSetViewRequest::SelectIPSet(name),
            });

        let ipset_entries = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |msg| match msg {
                IPSetEntryItemResponse::RemoveEntry { entry } => {
                    IPSetViewRequest::RemoveEntry(entry)
                }
            });

        let model = Self {
            broker,
            ipset_dialog,
            ipsets,
            ipset_entries,
            ipset_list: Vec::new(),
            selected_ipset: None,
            ipset_details: None,
            entry_input: String::new(),
            entry_valid: false,
            entry_error: None,
            details_loading: false,
            tracker: 0,
        };

        let ipset_list_box = model.ipsets.widget();
        let ipset_entries_list_box = model.ipset_entries.widget();
        let widgets = view_output!();

        // Load initial data
        sender.input(IPSetViewRequest::LoadIPSets);

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: IPSetViewRequest, sender: AsyncComponentSender<Self>) {
        self.reset();
        match msg {
            IPSetViewRequest::LoadIPSets => {
                let broker = self.broker;
                let sender = sender.clone();
                relm4::spawn(async move {
                    match broker.get_ipsets().await {
                        Ok(ipsets) => {
                            sender.input(IPSetViewRequest::UpdateIPSets(ipsets));
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(
                                &e,
                                "Failed to load IP sets",
                            );
                        }
                    }
                });
            }
            IPSetViewRequest::UpdateIPSets(ipsets) => {
                let previous_selection = self.selected_ipset.clone();

                self.set_ipset_list(ipsets.clone());
                let mut list = self.ipsets.guard();
                list.clear();
                for ipset_name in &self.ipset_list {
                    list.push_back(ipset_name.clone());
                }
                drop(list);

                glib::g_log!(
                    LogLevel::Debug,
                    "Updated IP sets: {} items",
                    self.ipset_list.len()
                );

                if let Some(selected) = previous_selection {
                    if self.ipset_list.contains(&selected) {
                        self.set_selected_ipset(Some(selected.clone()));
                        self.set_ipset_details(None);
                        self.set_entry_input(String::new());
                        self.set_entry_valid(false);
                        self.set_entry_error(None);
                        self.set_details_loading(true);
                        self.ipset_entries.guard().clear();
                        sender.input(IPSetViewRequest::LoadIPSetDetails(selected));
                    } else {
                        self.clear_selection_state();
                    }
                } else {
                    self.clear_selection_state();
                }
            }
            IPSetViewRequest::ShowCreateDialog => {
                if let Some(window) = relm4::main_adw_application().active_window() {
                    self.ipset_dialog.widget().present(Some(&window));
                }
            }
            IPSetViewRequest::CreateIPSet(settings) => {
                let broker = self.broker;
                let sender = sender.clone();
                let name = settings.name.clone();
                relm4::spawn(async move {
                    match broker.create_ipset(settings).await {
                        Ok(_) => {
                            glib::g_log!(LogLevel::Message, "Created IP set: {}", name);
                            sender.input(IPSetViewRequest::LoadIPSets);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(
                                &e,
                                "Failed to create IP set",
                            );
                        }
                    }
                });
            }
            IPSetViewRequest::DeleteIPSet(name) => {
                if self.selected_ipset.as_deref() == Some(name.as_str()) {
                    self.clear_selection_state();
                }

                let broker = self.broker;
                let sender = sender.clone();
                relm4::spawn(async move {
                    match broker.delete_ipset(&name).await {
                        Ok(_) => {
                            glib::g_log!(LogLevel::Message, "Deleted IP set: {}", name);
                            sender.input(IPSetViewRequest::LoadIPSets);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(
                                &e,
                                "Failed to delete IP set",
                            );
                        }
                    }
                });
            }
            IPSetViewRequest::SelectIPSet(name) => {
                if self.selected_ipset.as_deref() != Some(name.as_str()) {
                    self.set_selected_ipset(Some(name.clone()));
                    self.set_ipset_details(None);
                    self.set_entry_input(String::new());
                    self.set_entry_valid(false);
                    self.set_entry_error(None);
                    self.set_details_loading(true);
                    self.ipset_entries.guard().clear();
                    sender.input(IPSetViewRequest::LoadIPSetDetails(name));
                }
            }
            IPSetViewRequest::LoadIPSetDetails(name) => {
                let broker = self.broker;
                let sender = sender.clone();
                relm4::spawn(async move {
                    match broker.get_ipset_details(&name).await {
                        Ok(details) => {
                            sender.input(IPSetViewRequest::UpdateIPSetDetails(details));
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(
                                &e,
                                "Failed to load IP set details",
                            );
                            sender.input(IPSetViewRequest::LoadIPSetDetailsFailed);
                        }
                    }
                });
            }
            IPSetViewRequest::UpdateIPSetDetails(settings) => {
                self.set_details_loading(false);
                if self
                    .selected_ipset
                    .as_deref()
                    .map(|current| current == settings.name.as_str())
                    .unwrap_or(false)
                {
                    let entries = settings.entries.clone();
                    self.set_ipset_details(Some(settings));
                    self.set_entry_input(String::new());
                    self.set_entry_valid(false);
                    self.set_entry_error(None);

                    let mut list = self.ipset_entries.guard();
                    list.clear();
                    for entry in entries {
                        list.push_back(entry);
                    }
                }
            }
            IPSetViewRequest::LoadIPSetDetailsFailed => {
                self.set_details_loading(false);
            }
            IPSetViewRequest::UpdateEntryInput(value) => {
                self.set_entry_input(value.clone());

                if value.trim().is_empty() {
                    self.set_entry_valid(false);
                    self.set_entry_error(None);
                } else if let Some(details) = self.ipset_details.as_ref() {
                    match validate_ipset_entry(&value, &details.ipset_type) {
                        Ok(_) => {
                            self.set_entry_valid(true);
                            self.set_entry_error(None);
                        }
                        Err(e) => {
                            self.set_entry_valid(false);
                            self.set_entry_error(Some(e.to_string()));
                        }
                    }
                } else {
                    self.set_entry_valid(false);
                    self.set_entry_error(None);
                }
            }
            IPSetViewRequest::AddEntry => {
                if !self.entry_valid {
                    return;
                }

                if let Some(ipset_name) = self.selected_ipset.clone() {
                    let entry = self.entry_input.trim().to_string();
                    if entry.is_empty() {
                        return;
                    }

                    self.set_entry_input(String::new());
                    self.set_entry_valid(false);
                    self.set_entry_error(None);
                    self.set_details_loading(true);

                    let broker = self.broker;
                    let sender = sender.clone();
                    relm4::spawn(async move {
                        match broker.add_ipset_entry(&ipset_name, &entry).await {
                            Ok(_) => {
                                sender.input(IPSetViewRequest::LoadIPSetDetails(ipset_name));
                            }
                            Err(e) => {
                                crate::core::error_handling::async_helpers::log_error(
                                    &e,
                                    "Failed to add IP set entry",
                                );
                                sender.input(IPSetViewRequest::LoadIPSetDetailsFailed);
                            }
                        }
                    });
                }
            }
            IPSetViewRequest::RemoveEntry(entry) => {
                if let Some(ipset_name) = self.selected_ipset.clone() {
                    self.set_details_loading(true);
                    let broker = self.broker;
                    let sender = sender.clone();
                    relm4::spawn(async move {
                        match broker.remove_ipset_entry(&ipset_name, &entry).await {
                            Ok(_) => {
                                sender.input(IPSetViewRequest::LoadIPSetDetails(ipset_name));
                            }
                            Err(e) => {
                                crate::core::error_handling::async_helpers::log_error(
                                    &e,
                                    "Failed to remove IP set entry",
                                );
                                sender.input(IPSetViewRequest::LoadIPSetDetailsFailed);
                            }
                        }
                    });
                }
            }
        }
    }
}

impl IPSetView {
    fn clear_selection_state(&mut self) {
        self.set_selected_ipset(None);
        self.set_ipset_details(None);
        self.set_entry_input(String::new());
        self.set_entry_valid(false);
        self.set_entry_error(None);
        self.set_details_loading(false);

        let mut entries = self.ipset_entries.guard();
        entries.clear();
    }

    fn selected_ipset_type(&self) -> &str {
        self.ipset_details
            .as_ref()
            .map(|settings| settings.ipset_type.as_str())
            .unwrap_or("")
    }

    fn selected_ipset_entry_count(&self) -> usize {
        self.ipset_details
            .as_ref()
            .map(|settings| settings.entries.len())
            .unwrap_or(0)
    }

    fn selected_ipset_option_count(&self) -> usize {
        self.ipset_details
            .as_ref()
            .map(|settings| settings.options.len())
            .unwrap_or(0)
    }
}
