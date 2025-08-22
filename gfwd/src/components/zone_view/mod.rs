mod port_item;
mod view_mode;

use relm4::actions::{AccelsPlus, ActionGroupName, RelmAction, RelmActionGroup};
use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::components::zone_view::view_mode::{ZoneViewModeMsg, ZoneViewModeOut};
use crate::fwd_broker::FwdBroker;
use crate::fwd_broker::ZoneSettings;
use view_mode::ZoneInfoComponent;

#[tracker::track]
pub struct ZoneView {
    #[tracker::do_not_track]
    broker: &'static FwdBroker,
    // child components
    #[tracker::do_not_track]
    view_mode: Controller<ZoneInfoComponent>,
    // props
    current_zone_name: String,
    firewalld_running: bool,
}

/// Requests that can be sent to the ZoneView component.
#[derive(Debug)]
pub enum ZoneViewRequest {
    SetZoneContent(String),
    ToggleFirewalld,
    UpdateZoneSettings(ZoneSettings),
    RemoveZone,
    SetFirewalldRunning(bool),
    AddPort(String, String),
    AddForwardPort(String, String, String, String), // port, protocol, to_port, to_addr
    RemovePort(String, String),
}

/// Responses that can be emitted from the ZoneView component.
#[derive(Debug)]
pub enum ZoneViewResponse {
    ToggleSidebar,
    RemovedZoneSuccess(String),
}

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(ActivateZoneAction, WindowActionGroup, "win.aza");
relm4::new_stateless_action!(RemoveZoneAction, WindowActionGroup, "win.rza");

#[relm4::component(async, pub)]
impl AsyncComponent for ZoneView {
    type Init = (String, gtk::ApplicationWindow);
    type Input = ZoneViewRequest;
    type Output = ZoneViewResponse;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[name = "title_bar"]
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

                pack_end = &gtk::MenuButton {
                    #[wrap(Some)]
                    set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {}
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

                    // Always show ZoneViewMode
                    #[local_ref]
                    zone_info -> adw::PreferencesPage {}
                },
            },
        }
    }

    menu! {
        main_menu: {
            "Activate Zone" => ActivateZoneAction,
            "Delete Zone" => RemoveZoneAction,
        }
    }

    async fn init(
        (initial_zone_name, app_window): Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;

        // Fetch initial zone settings
        let initial_settings = match broker.get_zone_settings(&initial_zone_name).await {
            Ok(settings) => Some(settings),
            Err(e) => {
                glib::g_log!(
                    LogLevel::Message,
                    "Failed to get initial zone settings: {}",
                    e
                );
                None
            }
        };

        let view_mode = ZoneInfoComponent::builder().launch(()).forward(
            sender.input_sender(),
            |out| match out {
                ZoneViewModeOut::AddPort { port, protocol, forward_port } => {
                    if let Some(forward) = forward_port {
                        ZoneViewRequest::AddForwardPort(port, protocol, forward.to_port, forward.to_addr)
                    } else {
                        ZoneViewRequest::AddPort(port, protocol)
                    }
                }
                ZoneViewModeOut::RemovePort(port, protocol) => {
                    ZoneViewRequest::RemovePort(port, protocol)
                }
            },
        );

        // Only emit settings if they were successfully fetched
        if let Some(settings) = &initial_settings {
            view_mode.emit(ZoneViewModeMsg::SetSettings(settings.clone()));
        }

        let initial_firewalld_running = broker.is_firewalld_active().await.unwrap_or(false);

        let model = ZoneView {
            broker,
            view_mode,
            current_zone_name: initial_zone_name,
            firewalld_running: initial_firewalld_running,
            tracker: 0,
        };

        let zone_info = model.view_mode.widget();
        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_accelerators_for_action::<ActivateZoneAction>(&["<primary>A"]);
        app.set_accelerators_for_action::<RemoveZoneAction>(&["<primary>D"]);

        let sender_c1 = sender.clone();
        let activate_zone_action: RelmAction<ActivateZoneAction> = {
            let action = RelmAction::new_stateless(move |_| {
                let _ = sender_c1;
                glib::g_log!(LogLevel::Info, "ActivateZoneAction triggered");
            });
            action.set_enabled(true);
            action
        };
        let sender_c2 = sender.clone();
        let remove_zone_action: RelmAction<RemoveZoneAction> = {
            let action = RelmAction::new_stateless(move |_| {
                sender_c2.input(ZoneViewRequest::RemoveZone);
            });
            action.set_enabled(true);
            action
        };

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(activate_zone_action);
        group.add_action(remove_zone_action);
        app_window.insert_action_group(WindowActionGroup::NAME, Some(&group.into_action_group()));

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();
        match msg {
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
                self.view_mode.emit(ZoneViewModeMsg::SetSettings(settings));
            }
            ZoneViewRequest::RemoveZone => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender = sender.clone();
                relm4::spawn(async move {
                    match broker.remove_zone(zone_name.as_str()).await {
                        Ok(()) => {
                            glib::g_log!(LogLevel::Info, "Stateless action for deleting zone!");
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
                        .add_forward_port(zone_name.as_str(), port.as_str(), protocol.as_str(), to_port.as_str(), to_addr.as_str())
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
        }
    }
}
