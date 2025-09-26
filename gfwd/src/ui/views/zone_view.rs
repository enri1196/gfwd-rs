use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::core::FwdBroker;
use crate::messages::zone::{ZoneViewRequest, ZoneViewResponse};
use crate::models::ZoneSettings;
use crate::ui::components::{PortItem, PortItemResponse};
use crate::ui::dialogs::AddPortDialog;
use crate::messages::port::PortDialogResponse;

#[tracker::track]
pub struct ZoneView {
    #[tracker::do_not_track]
    broker: &'static FwdBroker,
    #[tracker::do_not_track]
    port_dialog: AsyncController<AddPortDialog>,
    #[tracker::do_not_track]
    ports: FactoryVecDeque<PortItem>,
    current_zone_name: String,
    firewalld_running: bool,
}

#[relm4::component(async, pub)]
impl AsyncComponent for ZoneView {
    type Init = (String, adw::ApplicationWindow);
    type Input = ZoneViewRequest;
    type Output = ZoneViewResponse;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::HeaderBar {
                set_css_classes: &["flat"],

                pack_start = &gtk::Box {
                    set_spacing: 6,
                    gtk::Button {
                        set_icon_name: "sidebar-show-symbolic",
                        connect_clicked[sender] => move |_| {
                            sender.output(ZoneViewResponse::ToggleSidebar).unwrap();
                        },
                    },
                    gtk::Button {
                        #[track(model.changed(ZoneView::firewalld_running()))]
                        set_icon_name: if model.firewalld_running { "media-playback-stop-symbolic" } else { "media-playback-start-symbolic" },
                        #[track(model.changed(ZoneView::firewalld_running()))]
                        set_tooltip_text: Some(if model.firewalld_running { "Stop Firewalld" } else { "Start Firewalld" }),
                        connect_clicked[sender] => move |_| {
                            sender.input(ZoneViewRequest::ToggleFirewalld);
                        }
                    }
                },
            },

            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 12,
                    set_spacing: 12,

                    adw::PreferencesPage {
                        add = &adw::PreferencesGroup {
                            set_title: "General",
                            set_description: Some("Basic zone information"),

                            add = &adw::ActionRow {
                                set_title: "Name",
                                #[track(model.changed(ZoneView::current_zone_name()))]
                                set_subtitle: &model.current_zone_name,
                            },
                        },

                        add = &adw::PreferencesGroup {
                            set_title: "Allowed Ports",

                            #[wrap(Some)]
                            set_header_suffix = &gtk::Button {
                                set_icon_name: "list-add-symbolic",
                                set_tooltip_text: Some("Add port"),
                                add_css_class: "flat",
                                connect_clicked[sender] => move |_| {
                                    sender.input(ZoneViewRequest::ShowAddPortDialog);
                                },
                            },

                            #[local_ref]
                            ports_list_box -> gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::None,
                                add_css_class: "boxed-list",
                            },
                        },
                    },
                },
            },
        }
    }

    async fn init(
        (initial_zone_name, _app_window): Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;

        let port_dialog = AddPortDialog::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                PortDialogResponse::PortAdded { port, protocol, forwarding } => {
                    if let Some(forward) = forwarding {
                        ZoneViewRequest::AddForwardPort(port, protocol, forward.to_port, forward.to_addr)
                    } else {
                        ZoneViewRequest::AddPort(port, protocol)
                    }
                }
            });

        let ports = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |msg| match msg {
                PortItemResponse::RemovePort { port, protocol, forwarding } => {
                    if let Some(forward) = forwarding {
                        ZoneViewRequest::RemoveForwardPort(port, protocol, forward.to_port, forward.to_addr)
                    } else {
                        ZoneViewRequest::RemovePort(port, protocol)
                    }
                }
            });

        let initial_firewalld_running = broker.is_firewalld_active().await.unwrap_or(false);

        // Load initial zone settings
        let initial_zone_name_clone = initial_zone_name.clone();
        let sender_clone = sender.clone();
        relm4::spawn(async move {
            if let Ok(settings) = broker.get_zone_settings(&initial_zone_name_clone).await {
                sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
            }
        });

        let model = ZoneView {
            broker,
            port_dialog,
            ports,
            current_zone_name: initial_zone_name,
            firewalld_running: initial_firewalld_running,
            tracker: 0,
        };

        let ports_list_box = model.ports.widget();
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.reset();
        match msg {
            ZoneViewRequest::ShowAddPortDialog => {
                self.port_dialog.widget().present(Some(root));
            }
            ZoneViewRequest::AddPort(port, protocol) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    let _ = broker
                        .add_port(zone_name.as_str(), port.as_str(), protocol.as_str())
                        .await;
                    if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                        let _ = sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                    }
                });
            }
            ZoneViewRequest::SetZoneContent(zone_name) => {
                self.set_current_zone_name(zone_name.clone());
                let sender = sender.clone();
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                relm4::spawn(async move {
                    match broker.get_zone_settings(&zone_name).await {
                        Ok(settings) => {
                            sender.input(ZoneViewRequest::UpdateZoneSettings(settings));
                        }
                        Err(e) => {
                            glib::g_log!(LogLevel::Error, "Failed to update zone content: {e}");
                        }
                    }
                });
            }
            ZoneViewRequest::UpdateZoneSettings(settings) => {
                // Update port list
                let mut ports = self.ports.guard();
                ports.clear();

                // Add regular ports
                for (port, protocol) in &settings.ports {
                    ports.push_back(PortItem::from((port.clone(), protocol.clone())));
                }

                // Add forwarded ports
                for (port, protocol, to_port, to_addr) in &settings.forward_ports {
                    ports.push_back(PortItem::from((
                        port.clone(),
                        protocol.clone(),
                        to_port.clone(),
                        to_addr.clone(),
                    )));
                }

                glib::g_log!(LogLevel::Message, "Zone settings updated with {} ports", ports.len());
            }
            ZoneViewRequest::RemoveZone => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender = sender.clone();
                relm4::spawn(async move {
                    match broker.remove_zone(zone_name.as_str()).await {
                        Ok(()) => {
                            glib::g_log!(LogLevel::Info, "Zone deleted successfully");
                            let _ = sender
                                .output(ZoneViewResponse::RemovedZoneSuccess(zone_name.clone()));
                        }
                        Err(e) => glib::g_log!(LogLevel::Error, "Could not delete zone: {e}"),
                    };
                });
            }
            ZoneViewRequest::SetFirewalldRunning(is_running) => {
                self.set_firewalld_running(is_running);
            }
            ZoneViewRequest::ToggleFirewalld => {
                let desired_start = !self.get_firewalld_running();
                let broker = self.broker;
                let sender = sender.clone();
                self.set_firewalld_running(desired_start);
                relm4::spawn(async move {
                    let result = if desired_start {
                        broker.start_firewalld().await
                    } else {
                        broker.stop_firewalld().await
                    };
                    if let Err(e) = result {
                        glib::g_log!(LogLevel::Error, "Failed to toggle firewalld: {:?}", e);
                    }
                    let is_running = broker.is_firewalld_active().await.unwrap_or(false);
                    let _ = sender.input(ZoneViewRequest::SetFirewalldRunning(is_running));
                });
            }
            ZoneViewRequest::AddForwardPort(port, protocol, to_port, to_addr) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    let _ = broker
                        .add_forward_port(
                            zone_name.as_str(),
                            port.as_str(),
                            protocol.as_str(),
                            to_port.as_str(),
                            to_addr.as_str(),
                        )
                        .await;
                    if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                        let _ = sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                    }
                });
            }
            ZoneViewRequest::RemovePort(port, protocol) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    let _ = broker
                        .remove_port(zone_name.as_str(), port.as_str(), protocol.as_str())
                        .await;
                    if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                        let _ = sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                    }
                });
            }
            ZoneViewRequest::RemoveForwardPort(port, protocol, to_port, to_addr) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    if let Err(err) = broker
                        .remove_forward_port(
                            zone_name.as_str(),
                            port.as_str(),
                            protocol.as_str(),
                            to_port.as_str(),
                            to_addr.as_str(),
                        )
                        .await
                    {
                        glib::g_log!(LogLevel::Error, "Could not remove forward port: {}", err);
                    } else {
                        if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                            let _ =
                                sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                        }
                    }
                });
            }
        }
    }
}