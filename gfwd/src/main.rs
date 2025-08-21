mod components;
mod error;
mod fwd_broker;

use std::rc::Rc;

use relm4::adw::{ApplicationWindow, prelude::*};
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::components::sidebar::{SidebarView, SidebarViewRequest};
// use crate::components::zone_content::{ZoneView, ZoneViewRequest};
use crate::components::zone_dialog::{AddZoneDialog, AddZoneDialogResponse};
use crate::components::zone_view::{ZoneView, ZoneViewRequest};
use crate::fwd_broker::FwdBroker;

#[tracker::track]
struct App {
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
    sidebar_visible: bool,
}

#[derive(Debug)]
enum AppRequest {
    ToggleSidebar,
    ShowAddZoneDialog,
    ZoneAdded(AddZoneDialogResponse),
    ZoneRemoved(String),
    UpdateContentWithZoneName(String),
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for App {
    type Init = ();
    type Input = AppRequest;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_size: (1280, 720),

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

    async fn update(&mut self, msg: AppRequest, _sender: AsyncComponentSender<Self>) {
        self.reset();
        match msg {
            AppRequest::ToggleSidebar => {
                self.set_sidebar_visible(!self.sidebar_visible);
            }
            AppRequest::ShowAddZoneDialog => {
                self.dialog.widget().present(Some(self.root.as_ref()));
            }
            AppRequest::ZoneAdded(AddZoneDialogResponse::ZoneSettings(settings))
                if !settings.name.is_empty() =>
            {
                let zone_name = settings.name.to_string();
                match self.broker.add_zone(settings).await {
                    Ok(_) => {
                        glib::g_log!(LogLevel::Message, "Created new Zone: {}", zone_name);
                        self.sidebar.emit(SidebarViewRequest::UpdateZones)
                    }
                    Err(e) => glib::g_log!(LogLevel::Error, "Failed to add zone: {}", e),
                };
            }
            AppRequest::ZoneAdded(AddZoneDialogResponse::ZoneSettings(_)) => {
                glib::g_log!(LogLevel::Error, "Failed to add zone");
            }
            AppRequest::ZoneRemoved(removed_zone) => {
                self.sidebar.emit(SidebarViewRequest::RemoveZone(removed_zone));
                let active_zone = self.broker.get_default_zone().await.unwrap();
                self.zone_view
                    .emit(ZoneViewRequest::SetZoneContent(active_zone));
            }
            AppRequest::UpdateContentWithZoneName(zone_name) => {
                self.zone_view
                    .emit(ZoneViewRequest::SetZoneContent(zone_name));
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
                components::sidebar::SidebarViewResponse::ShowAddZoneDialog => {
                    AppRequest::ShowAddZoneDialog
                }
                components::sidebar::SidebarViewResponse::SelectedZone(item_name) => {
                    AppRequest::UpdateContentWithZoneName(item_name)
                }
            });

        let initial_zone_name: String = broker
            .get_default_zone()
            .await
            .unwrap_or(String::from("default"));

        let zone_view = ZoneView::builder()
            .launch((initial_zone_name, root.clone().into()))
            .forward(sender.input_sender(), |resp| match resp {
                components::zone_view::ZoneViewResponse::ToggleSidebar => AppRequest::ToggleSidebar,
                components::zone_view::ZoneViewResponse::RemovedZoneSuccess(removed_zone) => AppRequest::ZoneRemoved(removed_zone),
            });

        let dialog = AddZoneDialog::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| AppRequest::ZoneAdded(msg));

        let root = Rc::new(root);

        let model = App {
            broker,
            root: Rc::clone(&root),
            dialog,
            sidebar,
            zone_view,
            sidebar_visible: true,
            tracker: 0,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("com.github.Gfwd");
    app.run_async::<App>(());
}
