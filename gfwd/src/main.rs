mod fwd_broker;

mod components;
mod dialogs;

use std::convert::identity;

use relm4::adw::prelude::*;
use relm4::gtk::{self, glib};
use relm4::MessageBroker;
use relm4::prelude::*;

use crate::components::sidebar::view::SidebarView;
use crate::components::zone_view::ZoneView;

static DIALOG_BROKER: MessageBroker<DialogMsg> = MessageBroker::new();

struct Dialog {
    visible: bool,
}

#[derive(Debug)]
enum DialogMsg {
    Show,
    Hide,
}

#[relm4::component]
impl SimpleComponent for Dialog {
    type Init = ();
    type Input = DialogMsg;
    type Output = ButtonMsg;

    view! {
        dialog = gtk::AboutDialog {
            #[watch]
            set_visible: model.visible,
            set_modal: true,

            #[wrap(Some)]
            set_child = &gtk::Label {
                set_width_request: 200,
                set_height_request: 80,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                #[watch]
                set_label: if dialog.transient_for().is_some() {
                    "I'm transient!"
                } else {
                    "I'm not transient..."
                },
            },

            connect_close_request[sender] => move |_| {
                sender.input(DialogMsg::Hide);
                glib::Propagation::Stop
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Dialog { visible: false };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            DialogMsg::Show => self.visible = true,
            DialogMsg::Hide => self.visible = false,
        }
    }
}

struct Button {
    dialog: Controller<Dialog>,
}

#[derive(Debug)]
enum ButtonMsg {}

#[relm4::component]
impl SimpleComponent for Button {
    type Init = ();
    type Input = ButtonMsg;
    type Output = AppMsg;

    view! {
        button = &gtk::Button {
            set_label: "Show the dialog",
            connect_clicked => move |_| {
                DIALOG_BROKER.send(DialogMsg::Show);
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dialog = Dialog::builder()
            .transient_for(&root)
            .launch_with_broker((), &DIALOG_BROKER)
            .forward(sender.input_sender(), identity);

        let model = Button { dialog };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}

#[derive(Debug)]
enum AppMsg {
    ToggleSidebar,
}

#[tracker::track]
struct Visibility {
    sidebar_visible: bool,
}

struct App {
    visibility: Visibility,
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

            #[wrap(Some)]
            set_content = &adw::OverlaySplitView {
                #[track(model.visibility.changed(Visibility::sidebar_visible()))]
                set_show_sidebar: model.visibility.sidebar_visible,

                #[wrap(Some)]
                set_sidebar = model.sidebar.widget(),

                #[wrap(Some)]
                set_content = model.zone_view.widget(),
            }
        }
    }


    async fn update(&mut self, msg: AppMsg, _sender: AsyncComponentSender<Self>) {
        match msg {
            AppMsg::ToggleSidebar => {
                self.visibility
                    .set_sidebar_visible(!self.visibility.sidebar_visible);
            }
            // AppMsg::SidebarMsg(msg) => {
            //     self.sidebar.emit(msg);
            // }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let sidebar = SidebarView::builder()
            .launch(())
            .forward(sender.command_sender(), identity);

        let zone_view = ZoneView::builder()
            .launch("default".to_string())
            .forward(sender.input_sender(), |msg| AppMsg::ToggleSidebar);

        let model = App {
            visibility: Visibility {
                sidebar_visible: false,
                tracker: 0,
            },
            sidebar,
            zone_view,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }}

fn main() {
    let app = RelmApp::new("com.github.Gfwd");
    app.run_async::<App>(());
}
