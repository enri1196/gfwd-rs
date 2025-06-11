use gfwd_bus::config_firewalld1::ZoneSettings;
use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct AddZoneDialog {
    name: String,
    description: String,
    // You can add more fields here for other settings
}

#[derive(Debug)]
pub enum AddZoneDialogMsg {
    SetName(String),
    SetDescription(String),
    /// User clicked "Add"
    Add,
    /// User clicked "Cancel" or closed the dialog
    Cancel,
}

// The output of our dialog. The parent will receive this.
// We use an Option to signify if the user confirmed or cancelled.
pub struct AddZoneDialogOutput {
    pub name: String,
    pub settings: ZoneSettings,
}

impl std::fmt::Debug for AddZoneDialogOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AddZoneDialogOutput")
            .field("name", &self.name)
            // .field("settings", &self.settings)
            .finish()
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AddZoneDialog {
    type Init = ();
    type Input = AddZoneDialogMsg;
    type Output = AddZoneDialogOutput;
    type Widgets = AddZoneDialogWidgets;

    view! {
        dialog = adw::Dialog {
            set_title: "New Zone",
            // set_modal: true,
            // set_default_width: 500,
            // set_default_height: 300,

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                set_margin_all: 12,

                // Name field
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,
                    
                    append = &gtk::Label {
                        set_text: "Zone Name:",
                        set_halign: gtk::Align::Start,
                    },
                    
                    append = &gtk::Entry {
                        set_hexpand: true,
                        set_placeholder_text: Some("Enter zone name"),
                        // connect_changed: clone!(@strong sender => move |entry| {
                        //     if let Some(text) = entry.text().as_str() {
                        //         sender.input(AddZoneDialogMsg::SetName(text.to_string()));
                        //     }
                        // }),
                    },
                },

                // Description field
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,
                    
                    append = &gtk::Label {
                        set_text: "Description:",
                        set_halign: gtk::Align::Start,
                    },
                    
                    append = &gtk::Entry {
                        set_hexpand: true,
                        set_placeholder_text: Some("Enter description"),
                        // connect_changed: clone!(@strong sender => move |entry| {
                        //     if let Some(text) = entry.text().as_str() {
                        //         sender.input(AddZoneDialogMsg::SetDescription(text.to_string()));
                        //     }
                        // }),
                    },
                },

                // Action buttons
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::End,
                    set_spacing: 6,
                    set_margin_top: 12,

                    append = &gtk::Button::with_label("Cancel") {
                        connect_clicked: move |_| {
                            sender.input(AddZoneDialogMsg::Cancel);
                        },
                    },

                    append = &gtk::Button::with_label("Add") {
                        add_css_class: "suggested-action",
                        set_sensitive: false,
                        // connect_clicked: move |_| {
                        //     sender.input(AddZoneDialogMsg::Add);
                        // },
                    },
                },
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AddZoneDialog {
            name: String::new(),
            description: String::new(),
        };

        println!("SOME TEXT FROM DIALOG!");

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        // Construct the ZoneSettings tuple.
        // We fill in the known values and use defaults for the rest.
        let settings: ZoneSettings = (
            String::new(), // version
            self.name.clone(),
            self.description.clone(),
            false,                 // UNUSED
            "default".to_string(), // target
            Vec::new(),            // services
            Vec::new(),            // ports
            Vec::new(),            // icmp-blocks
            false,                 // masquerade
            Vec::new(),            // forward-ports
            Vec::new(),            // interfaces
            Vec::new(),            // sources
            Vec::new(),            // rich rules
            Vec::new(),            // protocols
            Vec::new(),            // source-ports
        );

        match msg {
            AddZoneDialogMsg::SetName(name) => {
                self.name = name;
                // Enable the "Add" button only if the name is not empty
                // sender.widgets().set_response_enabled("add", !self.name.is_empty());
            }
            AddZoneDialogMsg::SetDescription(desc) => {
                self.description = desc;
            }
            AddZoneDialogMsg::Add => {
                // Send the data back to the parent and destroy the dialog
                sender
                    .output(AddZoneDialogOutput {
                        name: self.name.clone(),
                        settings,
                    })
                    .unwrap();
            }
            AddZoneDialogMsg::Cancel => {
                // Send `None` back and destroy the dialog
                sender
                    .output(AddZoneDialogOutput {
                        name: String::new(),
                        settings,
                    })
                    .unwrap();
            }
        }
    }
}
