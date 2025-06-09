use gfwd_bus::config_firewalld1::ZoneSettings;
use relm4::adw::prelude::*;
use relm4::gtk::prelude::*;
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
            set_visible: true,

            #[wrap(Some)]
            set_child = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: "Basic Information",

                    add = &adw::EntryRow {
                        set_title: "Name",
                        set_tooltip_text: Some("e.g., my-custom-zone"),
                        set_text: &model.name,
                        connect_text_notify[sender] => move |entry| {
                            sender.input(AddZoneDialogMsg::SetName(entry.text().to_string()));
                        }
                    },

                    add = &adw::EntryRow {
                        set_title: "Description",
                        set_tooltip_text: Some("A short description of the zone"),
                        set_text: &model.description,
                        connect_text_notify[sender] => move |entry| {
                            sender.input(AddZoneDialogMsg::SetDescription(entry.text().to_string()));
                        }
                    },
                }
            },

            // // Add response buttons for "Cancel" and "Add"
            // add_response: ("cancel", "Cancel"),
            // add_response: ("add", "Add"),

            // // Set the default and suggested actions
            // set_response_enabled: ("add", false), // Initially disabled
            // set_default_response: Some("add"),
            // set_close_response: "cancel",

            // connect_response[sender] => move |_, response| {
            //     match response {
            //         "add" => sender.input(AddZoneDialogMsg::Add),
            //         _ => sender.input(AddZoneDialogMsg::Cancel),
            //     }
            // }
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
