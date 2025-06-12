mod fwd_broker;

mod components;

use std::convert::identity;

use relm4::adw::prelude::*;
use relm4::{prelude::*, MessageBroker};

use crate::components::sidebar::{InputSidebarMsg, SidebarView};
use crate::components::zone_content::ZoneView;
use crate::components::zone_dialog::{AddZoneDialog, AddZoneDialogOutput};
use crate::fwd_broker::FwdBroker;

#[derive(Debug)]
enum AppMsg {
    ToggleSidebar,
    ShowAddZoneDialog,
    ZoneAdded(AddZoneDialogOutput),
}

#[tracker::track]
struct Visibility {
    sidebar_visible: bool,
}

struct App {
    broker: &'static FwdBroker,
    visibility: Visibility,
    dialog: AsyncController<AddZoneDialog>,
    sidebar: AsyncController<SidebarView>,
    zone_view: Controller<ZoneView>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for App {
    type Init = ();
    type Input = AppMsg;
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

    async fn update(&mut self, msg: AppMsg, _sender: AsyncComponentSender<Self>) {
        match msg {
            AppMsg::ToggleSidebar => {
                self.visibility
                    .set_sidebar_visible(!self.visibility.sidebar_visible);
            }
            AppMsg::ShowAddZoneDialog => {
                self.dialog.widget().present(None::<&gtk::Box>);
            }
            AppMsg::ZoneAdded(output) => {
                if !output.settings.name.is_empty() {
                    match self.broker.add_zone(output.settings).await {
                        Ok(_) => self.sidebar.emit(InputSidebarMsg::UpdateZones),
                        Err(e) => println!("Failed to add zone: {}", e),
                    };
                }
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
            .forward(sender.input_sender(), |_| AppMsg::ShowAddZoneDialog);

        let zone_view = ZoneView::builder()
            .launch("default".to_string())
            .forward(sender.input_sender(), |_| AppMsg::ToggleSidebar);

        let model = App {
            visibility: Visibility {
                sidebar_visible: false,
                tracker: 0,
            },
            dialog: AddZoneDialog::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| AppMsg::ZoneAdded(msg)),
            sidebar,
            zone_view,
            broker: FwdBroker::get_broker().await,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }}

fn main() {
    let app = RelmApp::new("com.github.Gfwd");
    app.run_async::<App>(());
}
