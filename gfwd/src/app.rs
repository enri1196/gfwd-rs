use std::rc::Rc;

use relm4::abstractions::Toaster;
use relm4::adw::{ApplicationWindow, prelude::*};
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::core::{FwdBroker};
use crate::messages::{AppRequest, SidebarRequest, SidebarResponse, ZoneViewRequest, ZoneViewResponse};




use crate::ui::components::{show_toast, ToastMessage};
use crate::ui::dialogs::AddZoneDialog;
use crate::ui::views::{SidebarView, ZoneView};
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
    toaster: Toaster,
    sidebar_visible: bool,
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
                    set_content = model.zone_view.widget(),
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
                        show_toast(&self.toaster, ToastMessage::success(format!("Zone '{}' created successfully", zone_name)));
                        self.sidebar.emit(SidebarRequest::UpdateZones)
                    }
                    Err(e) => {
                        glib::g_log!(LogLevel::Error, "Failed to add zone: {}", e);
                        show_toast(&self.toaster, ToastMessage::error(e.user_message()));
                    }
                };
            }
            AppRequest::ZoneAdded(_) => {
                glib::g_log!(LogLevel::Error, "Failed to add zone");
                show_toast(&self.toaster, ToastMessage::error("Failed to add zone: Invalid zone name"));
            }
            AppRequest::ZoneRemoved(removed_zone) => {
                show_toast(&self.toaster, ToastMessage::success(format!("Zone '{}' deleted successfully", removed_zone)));
                self.sidebar.emit(SidebarRequest::RemoveZone(removed_zone));
                let active_zone = self.broker.get_default_zone().await.unwrap();
                self.zone_view.emit(ZoneViewRequest::SetZoneContent(active_zone));
            }
            AppRequest::UpdateContentWithZoneName(zone_name) => {
                self.zone_view.emit(ZoneViewRequest::SetZoneContent(zone_name));
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
                SidebarResponse::SelectedZone(item_name) => AppRequest::UpdateContentWithZoneName(item_name),
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
                ZoneViewResponse::RemovedZoneSuccess(removed_zone) => AppRequest::ZoneRemoved(removed_zone),
            });

        let dialog = AddZoneDialog::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                crate::messages::zone::ZoneDialogResponse::ZoneSettings(settings) => AppRequest::ZoneAdded(settings),
            });

        let toaster = Toaster::default();

        let model = App {
            broker,
            root: root_rc,
            dialog,
            sidebar,
            zone_view,
            toaster,
            sidebar_visible: true,
            tracker: 0,
        };

        let toast_overlay = model.toaster.overlay_widget();
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }
}