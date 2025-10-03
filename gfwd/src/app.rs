use std::rc::Rc;

use relm4::abstractions::Toaster;
use relm4::adw::{ApplicationWindow, prelude::*};
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::core::FwdBroker;
use crate::messages::{
    AppRequest, SidebarRequest, SidebarResponse, ZoneViewRequest, ZoneViewResponse,
};
use crate::messages::ipset::IPSetViewRequest;

// Toast components are now accessed through specific functions
use crate::ui::dialogs::AddZoneDialog;
use crate::ui::views::{SidebarView, ZoneView, IPSetView};
use crate::utils::constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};

#[tracker::track]
pub struct App {
    #[tracker::do_not_track]
    broker: &'static FwdBroker,
    #[tracker::do_not_track]
    root: Rc<ApplicationWindow>,
    #[tracker::do_not_track]
    dialog: AsyncController<AddZoneDialog>,
    #[tracker::do_not_track]
    sidebar: AsyncController<SidebarView>,
    #[tracker::do_not_track]
    zone_view: AsyncController<ZoneView>,
    #[tracker::do_not_track]
    ipset_view: AsyncController<IPSetView>,
    #[tracker::do_not_track]
    toaster: Toaster,
    sidebar_visible: bool,
    current_view: String,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for App {
    type Init = ();
    type Input = AppRequest;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_size: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),

            #[local_ref]
            toast_overlay -> adw::ToastOverlay {
                adw::OverlaySplitView {
                    #[track(model.changed(App::sidebar_visible()))]
                    set_show_sidebar: *model.get_sidebar_visible(),

                    #[wrap(Some)]
                    set_sidebar = model.sidebar.widget(),

                    #[wrap(Some)]
                    set_content = &gtk::Stack {
                        add_named[Some("zones")] = model.zone_view.widget(),
                        add_named[Some("ipsets")] = model.ipset_view.widget(),
                        #[track(model.changed(App::current_view()))]
                        set_visible_child_name: &model.current_view,
                    },
                }
            }
        }
    }

    async fn update(&mut self, msg: AppRequest, _sender: AsyncComponentSender<Self>) {
        self.reset();
        match msg {
            AppRequest::ToggleSidebar => {
                self.set_sidebar_visible(!self.sidebar_visible);
            }
            AppRequest::ShowAddZoneDialog => {
                self.dialog.widget().present(Some(self.root.as_ref()));
            }
            AppRequest::ZoneAdded(settings) if !settings.name.is_empty() => {
                let zone_name = settings.name.to_string();
                match self.broker.add_zone(settings).await {
                    Ok(_) => {
                        glib::g_log!(LogLevel::Message, "Created new Zone: {}", zone_name);
                        crate::ui::components::toast::show_success_toast(&self.toaster, "created", &format!("zone '{}'", zone_name));
                        self.sidebar.emit(SidebarRequest::UpdateZones)
                    }
                    Err(e) => {
                        crate::core::error_handling::async_helpers::log_error(&e, "Failed to add zone");
                        crate::ui::components::toast::show_error_toast(&self.toaster, &e);
                    }
                };
            }
            AppRequest::ZoneAdded(_) => {
                // This should not happen in normal operation
                let error = crate::core::error::GfwdError::zone_already_exists("unknown");
                crate::core::error_handling::async_helpers::log_error(&error, "Unexpected zone add failure");
                crate::ui::components::toast::show_error_toast(&self.toaster, &error);
            }
            AppRequest::ZoneRemoved(removed_zone) => {
                crate::ui::components::toast::show_success_toast(&self.toaster, "deleted", &format!("zone '{}'", removed_zone));
                self.sidebar.emit(SidebarRequest::RemoveZone(removed_zone));
                let active_zone = self.broker.get_default_zone().await.unwrap();
                self.zone_view
                    .emit(ZoneViewRequest::SetZoneContent(active_zone));
            }
            AppRequest::UpdateContentWithZoneName(zone_name) => {
                self.set_current_view("zones".to_string());
                self.zone_view
                    .emit(ZoneViewRequest::SetZoneContent(zone_name));
            }
            AppRequest::ShowIPSets => {
                self.set_current_view("ipsets".to_string());
                self.ipset_view.emit(IPSetViewRequest::LoadIPSets);
            }
            AppRequest::ShowZones => {
                self.set_current_view("zones".to_string());
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;

        let sidebar = SidebarView::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                SidebarResponse::ShowAddZoneDialog => AppRequest::ShowAddZoneDialog,
                SidebarResponse::SelectedZone(item_name) => {
                    AppRequest::UpdateContentWithZoneName(item_name)
                }
                SidebarResponse::ShowIPSets => AppRequest::ShowIPSets,
            });

        let initial_zone_name: String = broker
            .get_default_zone()
            .await
            .unwrap_or(String::from("default"));

        let root_rc = Rc::new(root.clone());

        let zone_view = ZoneView::builder()
            .launch((initial_zone_name, root_rc.as_ref().clone()))
            .forward(sender.input_sender(), |resp| match resp {
                ZoneViewResponse::ToggleSidebar => AppRequest::ToggleSidebar,
                ZoneViewResponse::RemovedZoneSuccess(removed_zone) => {
                    AppRequest::ZoneRemoved(removed_zone)
                }
            });

        let ipset_view = IPSetView::builder()
            .launch(broker)
            .forward(sender.input_sender(), |_resp| {
                // IPSetView doesn't emit responses that need app handling yet
                AppRequest::ToggleSidebar // Placeholder
            });

        let dialog = AddZoneDialog::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                crate::messages::zone::ZoneDialogResponse::ZoneSettings(settings) => {
                    AppRequest::ZoneAdded(settings)
                }
            });

        let toaster = Toaster::default();

        let model = App {
            broker,
            root: root_rc,
            dialog,
            sidebar,
            zone_view,
            ipset_view,
            toaster,
            sidebar_visible: true,
            current_view: "zones".to_string(),
            tracker: 0,
        };

        let toast_overlay = model.toaster.overlay_widget();
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }
}
