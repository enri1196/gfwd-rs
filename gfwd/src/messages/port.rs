use crate::models::ForwardingConfig;

#[derive(Debug)]
pub enum PortDialogRequest {
    SetPort(String),
    SetProtocol(String),
    SetIsForwarding(bool),
    SetDestIp(String),
    SetDestPort(String),
    ValidatePort,
    ValidateDestIp,
    ValidateDestPort,
    Add,
    Cancel,
}

#[derive(Debug)]
pub enum PortDialogResponse {
    PortAdded {
        port: String,
        protocol: String,
        forwarding: Option<ForwardingConfig>,
    },
}