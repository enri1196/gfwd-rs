// Example showcasing accessibility and styling improvements
// This file demonstrates the comprehensive styling and accessibility patterns
// implemented across the GFWD application

use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::ui::styling::{css_classes, icons};

/// Example dialog demonstrating accessibility and styling best practices
#[tracker::track]
#[derive(Debug)]
pub struct AccessibilityShowcaseDialog {
    sample_text: String,
    has_error: bool,
    selected_option: u32,
}

#[derive(Debug)]
pub enum ShowcaseRequest {
    SetText(String),
    ToggleError,
    SetOption(u32),
    Submit,
    Cancel,
}

#[derive(Debug)]
pub enum ShowcaseResponse {
    DataSubmitted {
        #[allow(dead_code)]
        text: String,
        #[allow(dead_code)]
        option: u32,
    },
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AccessibilityShowcaseDialog {
    type Init = ();
    type Input = ShowcaseRequest;
    type Output = ShowcaseResponse;

    view! {
        dialog = adw::Dialog {
            set_title: "Accessibility Showcase",
            set_content_width: 500,
            set_content_height: 600,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    add_css_class: css_classes::FLAT,

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Accessibility Showcase",
                        set_subtitle: "Demonstrating GFWD styling and accessibility patterns",
                    },

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        set_accessible_role: gtk::AccessibleRole::Button,
                        set_can_focus: true,
                        set_tooltip_text: Some("Cancel and close dialog"),
                        connect_clicked[sender, dialog] => move |_| {
                            sender.input(ShowcaseRequest::Cancel);
                            dialog.close();
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Submit",
                        add_css_class: css_classes::SUGGESTED_ACTION,
                        set_accessible_role: gtk::AccessibleRole::Button,
                        set_can_focus: true,
                        set_receives_default: true,
                        set_tooltip_text: Some("Submit form data"),
                        connect_clicked => ShowcaseRequest::Submit,
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,

                    adw::Clamp {
                        set_maximum_size: 450,
                        set_tightening_threshold: 400,

                        adw::PreferencesPage {
                            set_icon_name: Some(icons::PREFERENCES_SYSTEM),
                            set_title: "Accessibility Features",
                            set_description: "This dialog demonstrates comprehensive accessibility and styling patterns used throughout GFWD",

                            // Text Input Section
                            add = &adw::PreferencesGroup {
                                set_title: "Text Input with Validation",
                                set_description: Some("Demonstrates real-time validation with accessibility feedback"),

                                add = &adw::EntryRow {
                                    set_title: "Sample Text Field",
                                    set_text: &model.sample_text,
                                    set_accessible_role: gtk::AccessibleRole::TextBox,
                                    set_can_focus: true,
                                    #[track(model.changed(AccessibilityShowcaseDialog::has_error()))]
                                    add_css_class: if model.has_error { css_classes::ERROR } else { "" },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::EDIT),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                        set_tooltip_text: Some("Text input field"),
                                    },

                                    connect_changed[sender] => move |entry| {
                                        sender.input(ShowcaseRequest::SetText(entry.text().to_string()));
                                    },
                                },

                                // Validation error display
                                add = &adw::ActionRow {
                                    #[track(model.changed(AccessibilityShowcaseDialog::has_error()))]
                                    set_visible: model.has_error,
                                    set_title: "This field contains an error for demonstration",
                                    add_css_class: css_classes::ERROR,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::WARNING),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                        add_css_class: css_classes::ERROR,
                                    },
                                },

                                // Toggle error button
                                add = &adw::ActionRow {
                                    set_title: "Toggle Validation Error",
                                    set_subtitle: "Demonstrates error state styling",

                                    add_suffix = &gtk::Button {
                                        set_label: "Toggle",
                                        add_css_class: css_classes::FLAT,
                                        set_accessible_role: gtk::AccessibleRole::Button,
                                        set_can_focus: true,
                                        set_tooltip_text: Some("Toggle error state for demonstration"),
                                        connect_clicked => ShowcaseRequest::ToggleError,
                                    },
                                },
                            },

                            // Selection Section
                            add = &adw::PreferencesGroup {
                                set_title: "Selection Controls",
                                set_description: Some("Demonstrates accessible selection with keyboard navigation"),

                                add = &adw::ComboRow {
                                    set_title: "Sample Selection",
                                    set_subtitle: "Choose an option to demonstrate selection",
                                    set_accessible_role: gtk::AccessibleRole::ComboBox,
                                    set_can_focus: true,
                                    set_model: Some(&gtk::StringList::new(&[
                                        "Option 1 - Network Configuration",
                                        "Option 2 - Security Settings", 
                                        "Option 3 - Advanced Features",
                                    ])),
                                    #[track(model.changed(AccessibilityShowcaseDialog::selected_option()))]
                                    set_selected: model.selected_option,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::PREFERENCES_SYSTEM),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                    },

                                    connect_selected_notify[sender] => move |combo| {
                                        sender.input(ShowcaseRequest::SetOption(combo.selected()));
                                    },
                                },
                            },

                            // Status Indicators Section
                            add = &adw::PreferencesGroup {
                                set_title: "Status Indicators",
                                set_description: Some("Visual and accessible status communication"),

                                add = &adw::ActionRow {
                                    set_title: "Success Status",
                                    set_subtitle: "Indicates successful operations",
                                    set_accessible_role: gtk::AccessibleRole::ListItem,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::OK),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                        add_css_class: css_classes::SUCCESS,
                                        set_tooltip_text: Some("Success indicator"),
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Warning Status",
                                    set_subtitle: "Indicates caution or important information",
                                    set_accessible_role: gtk::AccessibleRole::ListItem,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::WARNING),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                        add_css_class: css_classes::WARNING,
                                        set_tooltip_text: Some("Warning indicator"),
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Error Status",
                                    set_subtitle: "Indicates errors or failures",
                                    set_accessible_role: gtk::AccessibleRole::ListItem,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::ERROR),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                        add_css_class: css_classes::ERROR,
                                        set_tooltip_text: Some("Error indicator"),
                                    },
                                },
                            },

                            // Action Buttons Section
                            add = &adw::PreferencesGroup {
                                set_title: "Action Button Styles",
                                set_description: Some("Different button styles with consistent accessibility"),

                                add = &adw::ActionRow {
                                    set_title: "Suggested Action",
                                    set_subtitle: "Primary action button styling",

                                    add_suffix = &gtk::Button {
                                        set_label: "Primary",
                                        add_css_class: css_classes::SUGGESTED_ACTION,
                                        set_accessible_role: gtk::AccessibleRole::Button,
                                        set_can_focus: true,
                                        set_tooltip_text: Some("Primary action button"),
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Destructive Action",
                                    set_subtitle: "Dangerous action button styling",

                                    add_suffix = &gtk::Button {
                                        set_label: "Delete",
                                        add_css_class: css_classes::FLAT,
                                        add_css_class: css_classes::DESTRUCTIVE_ACTION,
                                        set_accessible_role: gtk::AccessibleRole::Button,
                                        set_can_focus: true,
                                        set_tooltip_text: Some("Destructive action button"),
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Flat Button",
                                    set_subtitle: "Secondary action button styling",

                                    add_suffix = &gtk::Button {
                                        set_label: "Secondary",
                                        add_css_class: css_classes::FLAT,
                                        set_accessible_role: gtk::AccessibleRole::Button,
                                        set_can_focus: true,
                                        set_tooltip_text: Some("Secondary action button"),
                                    },
                                },
                            },

                            // Help and Information Section
                            add = &adw::PreferencesGroup {
                                set_title: "Accessibility Features",
                                set_description: Some("Summary of implemented accessibility improvements"),

                                add = &adw::ActionRow {
                                    set_title: "Keyboard Navigation",
                                    set_subtitle: "Full keyboard support with Tab, Arrow keys, Enter, and Escape",
                                    set_accessible_role: gtk::AccessibleRole::ListItem,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::INFO),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                        add_css_class: css_classes::ACCENT,
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Screen Reader Support",
                                    set_subtitle: "ARIA roles, labels, and descriptions for assistive technology",
                                    set_accessible_role: gtk::AccessibleRole::ListItem,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::INFO),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                        add_css_class: css_classes::ACCENT,
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Visual Indicators",
                                    set_subtitle: "Consistent color coding and iconography for status communication",
                                    set_accessible_role: gtk::AccessibleRole::ListItem,

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some(icons::INFO),
                                        set_pixel_size: 16,
                                        set_accessible_role: gtk::AccessibleRole::Img,
                                        add_css_class: css_classes::ACCENT,
                                    },
                                },
                            },
                        },
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
        let model = AccessibilityShowcaseDialog {
            sample_text: String::new(),
            has_error: false,
            selected_option: 0,
            tracker: 0,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();
        
        match msg {
            ShowcaseRequest::SetText(text) => {
                self.set_sample_text(text);
            }
            ShowcaseRequest::ToggleError => {
                self.set_has_error(!self.has_error);
            }
            ShowcaseRequest::SetOption(option) => {
                self.set_selected_option(option);
            }
            ShowcaseRequest::Submit => {
                sender
                    .output(ShowcaseResponse::DataSubmitted {
                        text: self.sample_text.clone(),
                        option: self.selected_option,
                    })
                    .unwrap();
            }
            ShowcaseRequest::Cancel => {
                // Reset form
                self.set_sample_text(String::new());
                self.set_has_error(false);
                self.set_selected_option(0);
            }
        }
    }
}