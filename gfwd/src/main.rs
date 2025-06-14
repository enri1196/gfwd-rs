mod components;
mod error;
mod fwd_broker;

use std::rc::Rc;

use relm4::adw::{prelude::*, ApplicationWindow};
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::components::sidebar::{SidebarViewRequest, SidebarView};
use crate::components::zone_content::{ZoneView, ZoneViewRequest};
use crate::components::zone_dialog::{AddZoneDialog, AddZoneDialogResponse};
use crate::fwd_broker::FwdBroker;

#[tracker::track]
struct Visibility {
    sidebar_visible: bool,
}

struct App {
    broker: &'static FwdBroker,
    root: Rc<ApplicationWindow>,
    visibility: Visibility,
    dialog: AsyncController<AddZoneDialog>,
    sidebar: AsyncController<SidebarView>,
    zone_view: AsyncController<ZoneView>,
}

#[derive(Debug)]
enum AppRequest {
    ToggleSidebar,
    ShowAddZoneDialog,
    ZoneAdded(AddZoneDialogResponse),
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

            adw::ToastOverlay {
                adw::OverlaySplitView {
                    #[track(model.visibility.changed(Visibility::sidebar_visible()))]
                    set_show_sidebar: model.visibility.sidebar_visible,

                    #[wrap(Some)]
                    set_sidebar = model.sidebar.widget(),

                    #[wrap(Some)]
                    set_content = model.zone_view.widget(),
                }
            }           
        }
    }

    async fn update(&mut self, msg: AppRequest, _sender: AsyncComponentSender<Self>) {
        match msg {
            AppRequest::ToggleSidebar => {
                self.visibility
                    .set_sidebar_visible(!self.visibility.sidebar_visible);
            }
            AppRequest::ShowAddZoneDialog => {
                self.dialog.widget().present(Some(self.root.as_ref()));
            }
            AppRequest::ZoneAdded(AddZoneDialogResponse::ZoneSettings(settings)) => {
                if !settings.name.is_empty() {
                    let zone_name = settings.name.to_string();
                    match self.broker.add_zone(settings).await {
                        Ok(_) => {
                            glib::g_log!(LogLevel::Message, "Created new Zone: {}", zone_name);
                            self.sidebar.emit(SidebarViewRequest::UpdateZones)
                        },
                        Err(e) => println!("Failed to add zone: {}", e),
                    };
                }
            },
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
        let sidebar = SidebarView::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                components::sidebar::SidebarViewResponse::ShowAddZoneDialog => AppRequest::ShowAddZoneDialog,
                components::sidebar::SidebarViewResponse::SelectedZone(item_name) => AppRequest::UpdateContentWithZoneName(item_name),
            });

        let zone_view = ZoneView::builder()
            .launch("default".to_string())
            .forward(sender.input_sender(), |_| AppRequest::ToggleSidebar);

        let dialog = AddZoneDialog::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| AppRequest::ZoneAdded(msg));

        let broker = FwdBroker::get_broker().await;

        let root = Rc::new(root);

        let model = App {
            visibility: Visibility {
                sidebar_visible: false,
                tracker: 0,
            },
            root: Rc::clone(&root),
            dialog,
            sidebar,
            zone_view,
            broker,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }}

fn main() {
    let app = RelmApp::new("com.github.Gfwd");
    app.run_async::<App>(());
}
