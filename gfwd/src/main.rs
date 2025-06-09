pub mod dialogs;
pub mod models;
pub mod sidebar;

use crate::sidebar::{SidebarMsg, SidebarView};
use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub enum AppMsg {
    ToggleSidebar,
    SidebarMsg(SidebarMsg), // Forward sidebar messages
}

#[tracker::track]
struct Visibility {
    sidebar_visible: bool,
}

struct AppModel {
    visibility: Visibility,
    sidebar: AsyncController<SidebarView>,
}

#[relm4::component(async)]
impl SimpleAsyncComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        adw::ApplicationWindow {
            set_default_width: 800,
            set_default_height: 600,

            #[wrap(Some)]
            set_content = &adw::OverlaySplitView {
                #[track(model.visibility.changed(Visibility::sidebar_visible()))]
                set_show_sidebar: model.visibility.sidebar_visible,

                #[wrap(Some)]
                set_sidebar = model.sidebar.widget(),

                #[wrap(Some)]
                set_content = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        set_show_title: false,

                        pack_start = &gtk::Button {
                            set_icon_name: "view-refresh-symbolic",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppMsg::ToggleSidebar);
                            },
                            set_halign: gtk::Align::Center,
                            set_hexpand: false,
                        },
                    },

                    #[wrap(Some)]
                    set_content = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                    },
                },
            }
        }
    }

    async fn update(&mut self, msg: AppMsg, _sender: AsyncComponentSender<Self>) {
        match msg {
            AppMsg::ToggleSidebar => {
                self.visibility
                    .set_sidebar_visible(!self.visibility.sidebar_visible);
            }
            AppMsg::SidebarMsg(msg) => {
                self.sidebar.emit(msg);
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
            .forward(sender.command_sender(), |msg| msg);

        let model = AppModel {
            visibility: Visibility {
                sidebar_visible: false,
                tracker: 0,
            },
            sidebar,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("com.github.Gfwd");
    app.run_async::<AppModel>(());
}
