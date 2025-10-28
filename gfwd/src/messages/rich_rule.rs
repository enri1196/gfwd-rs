#[derive(Debug, Clone)]
pub enum RichRuleDialogRequest {
    // Family selection
    SetFamily(String),

    // Source address
    SetSourceAddress(String),
    SetSourceInvert(bool),
    ValidateSource,

    // Destination address
    SetDestinationAddress(String),
    SetDestinationInvert(bool),
    ValidateDestination,

    // Service selection
    SetService(String),
    ValidateService,

    // Port specification
    SetPortNumber(String),
    SetPortProtocol(String),
    ValidatePort,

    // Protocol specification
    SetProtocol(String),
    ValidateProtocol,

    // Action selection
    SetAction(String),
    SetMarkValue(String),
    SetRejectType(String),
    ValidateAction,

    // Rule type selection (service, port, or protocol)
    SetRuleType(String),

    // Dialog actions
    Create,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum RichRuleDialogResponse {
    RichRuleCreated { rule_xml: String },
}
