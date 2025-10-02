use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::core::FwdBroker;
use crate::messages::ipset::{IPSetViewRequest, IPSetViewResponse};

use crate::ui::components::{IPSetItem, IPSetItemResponse};
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
    ipset_list: Vec<String>,
    selected_ipset: Option<String>,
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
                set_maximum_size: 800,
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
                            add_css_class: "flat",
                            connect_clicked => IPSetViewRequest::ShowCreateDialog,
                        },

                        #[local_ref]
                        ipset_list_box -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            #[track(model.changed(IPSetView::ipset_list()))]
                            set_visible: !model.ipset_list.is_empty(),
                        },

                        // Empty state
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
                }
            }
        }
    }

    async fn init(
        broker: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let ipset_dialog = IPSetDialog::builder()
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

        let model = Self {
            broker,
            ipset_dialog,
            ipsets,
            ipset_list: Vec::new(),
            selected_ipset: None,
            tracker: 0,
        };

        let ipset_list_box = model.ipsets.widget();
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
                            glib::g_log!(LogLevel::Error, "Failed to load IP sets: {}", e);
                        }
                    }
                });
            }
            IPSetViewRequest::UpdateIPSets(ipsets) => {
                self.set_ipset_list(ipsets.clone());
                let mut list = self.ipsets.guard();
                list.clear();
                for ipset_name in ipsets {
                    list.push_back(ipset_name);
                }
                glib::g_log!(LogLevel::Debug, "Updated IP sets: {} items", self.ipset_list.len());
            }
            IPSetViewRequest::ShowCreateDialog => {
                self.ipset_dialog.widget().present(Some(&relm4::main_adw_application().active_window().unwrap()));
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
                            glib::g_log!(LogLevel::Error, "Failed to create IP set: {}", e);
                        }
                    }
                });
            }
            IPSetViewRequest::DeleteIPSet(name) => {
                let broker = self.broker;
                let sender = sender.clone();
                relm4::spawn(async move {
                    match broker.delete_ipset(&name).await {
                        Ok(_) => {
                            glib::g_log!(LogLevel::Message, "Deleted IP set: {}", name);
                            sender.input(IPSetViewRequest::LoadIPSets);
                        }
                        Err(e) => {
                            glib::g_log!(LogLevel::Error, "Failed to delete IP set: {}", e);
                        }
                    }
                });
            }
            IPSetViewRequest::SelectIPSet(name) => {
                self.set_selected_ipset(Some(name));
            }
        }
    }
}