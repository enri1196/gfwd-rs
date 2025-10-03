use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug, Clone)]
pub struct RichRuleItem {
    rule_xml: String,
    display_text: String,
}

#[derive(Debug)]
pub enum RichRuleItemInput {
    Remove,
}

#[derive(Debug)]
pub enum RichRuleItemResponse {
    RemoveRichRule { rule_xml: String },
}

impl From<String> for RichRuleItem {
    fn from(rule_xml: String) -> Self {
        // Parse the XML to create a human-readable display text
        let display_text = Self::parse_rule_for_display(&rule_xml);
        Self {
            rule_xml,
            display_text,
        }
    }
}

impl RichRuleItem {
    /// Parse rich rule XML to create human-readable display text
    fn parse_rule_for_display(rule_xml: &str) -> String {
        // Simple XML parsing to extract key components for display
        let mut parts = Vec::new();
        
        // Extract family
        if rule_xml.contains("family=\"ipv4\"") {
            parts.push("IPv4".to_string());
        } else if rule_xml.contains("family=\"ipv6\"") {
            parts.push("IPv6".to_string());
        }
        
        // Extract source
        if let Some(start) = rule_xml.find("<source address=\"") {
            if let Some(end) = rule_xml[start + 17..].find("\"") {
                let address = &rule_xml[start + 17..start + 17 + end];
                let invert = rule_xml[start..].contains("invert=\"true\"");
                if invert {
                    parts.push(format!("NOT from {}", address));
                } else {
                    parts.push(format!("from {}", address));
                }
            }
        }
        
        // Extract destination
        if let Some(start) = rule_xml.find("<destination address=\"") {
            if let Some(end) = rule_xml[start + 22..].find("\"") {
                let address = &rule_xml[start + 22..start + 22 + end];
                let invert = rule_xml[start..].contains("invert=\"true\"");
                if invert {
                    parts.push(format!("NOT to {}", address));
                } else {
                    parts.push(format!("to {}", address));
                }
            }
        }
        
        // Extract service
        if let Some(start) = rule_xml.find("<service name=\"") {
            if let Some(end) = rule_xml[start + 15..].find("\"") {
                let service = &rule_xml[start + 15..start + 15 + end];
                parts.push(format!("service {}", service));
            }
        }
        
        // Extract port
        if let Some(start) = rule_xml.find("<port port=\"") {
            if let Some(end) = rule_xml[start + 12..].find("\"") {
                let port = &rule_xml[start + 12..start + 12 + end];
                if let Some(proto_start) = rule_xml[start..].find("protocol=\"") {
                    if let Some(proto_end) = rule_xml[proto_start + 10..].find("\"") {
                        let protocol = &rule_xml[proto_start + 10..proto_start + 10 + proto_end];
                        parts.push(format!("port {}/{}", port, protocol));
                    }
                } else {
                    parts.push(format!("port {}", port));
                }
            }
        }
        
        // Extract protocol
        if let Some(start) = rule_xml.find("<protocol value=\"") {
            if let Some(end) = rule_xml[start + 17..].find("\"") {
                let protocol = &rule_xml[start + 17..start + 17 + end];
                parts.push(format!("protocol {}", protocol));
            }
        }
        
        // Extract action
        let action = if rule_xml.contains("<accept/>") {
            "ACCEPT"
        } else if rule_xml.contains("<drop/>") {
            "DROP"
        } else if rule_xml.contains("<reject") {
            if let Some(start) = rule_xml.find("type=\"") {
                if let Some(end) = rule_xml[start + 6..].find("\"") {
                    let reject_type = &rule_xml[start + 6..start + 6 + end];
                    return format!("{} → REJECT ({})", parts.join(", "), reject_type);
                }
            }
            "REJECT"
        } else if let Some(start) = rule_xml.find("<mark set=\"") {
            if let Some(end) = rule_xml[start + 11..].find("\"") {
                let mark_value = &rule_xml[start + 11..start + 11 + end];
                return format!("{} → MARK ({})", parts.join(", "), mark_value);
            }
            "MARK"
        } else {
            "UNKNOWN"
        };
        
        if parts.is_empty() {
            format!("Any traffic → {}", action)
        } else {
            format!("{} → {}", parts.join(", "), action)
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for RichRuleItem {
    type Init = String;
    type Input = RichRuleItemInput;
    type Output = RichRuleItemResponse;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.display_text,
            set_subtitle: "Rich firewall rule",
            set_accessible_role: gtk::AccessibleRole::ListItem,
            add_prefix = &gtk::Image {
                set_icon_name: Some("applications-system-symbolic"),
                set_pixel_size: 16,
                set_accessible_role: gtk::AccessibleRole::Img,
            },
            add_suffix = &gtk::Button {
                set_icon_name: "user-trash-symbolic",
                set_tooltip_text: Some("Remove rich rule"),
                set_accessible_role: gtk::AccessibleRole::Button,
                set_can_focus: true,
                add_css_class: "flat",
                add_css_class: "destructive-action",
                set_valign: gtk::Align::Center,
                connect_clicked => RichRuleItemInput::Remove,
            },
        }
    }

    fn init_model(
        rule_xml: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self::from(rule_xml)
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            RichRuleItemInput::Remove => {
                sender
                    .output(RichRuleItemResponse::RemoveRichRule {
                        rule_xml: self.rule_xml.clone(),
                    })
                    .unwrap();
            }
        }
    }
}