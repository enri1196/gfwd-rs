use crate::models::IPSetSettings;

#[derive(Debug)]
pub enum IPSetViewRequest {
    LoadIPSets,
    UpdateIPSets(Vec<String>),
    ShowCreateDialog,
    CreateIPSet(IPSetSettings),
    DeleteIPSet(String),
    SelectIPSet(String),
    LoadIPSetDetails(String),
    UpdateIPSetDetails(IPSetSettings),
    UpdateEntryInput(String),
    AddEntry,
    RemoveEntry(String),
    LoadIPSetDetailsFailed,
}

#[derive(Debug)]
pub enum IPSetViewResponse {
    #[allow(dead_code)]
    IPSetSelected(String),
    #[allow(dead_code)]
    IPSetCreated(String),
    #[allow(dead_code)]
    IPSetDeleted(String),
}

#[derive(Debug)]
pub enum IPSetDialogRequest {
    SetName(String),
    ValidateName,
    SetType(String),
    ValidateType,
    SetCurrentEntry(String),
    ValidateCurrentEntry,
    AddEntry,
    #[allow(dead_code)]
    RemoveEntry(String),
    Create,
    Cancel,
}

#[derive(Debug)]
pub enum IPSetDialogResponse {
    IPSetCreated { settings: IPSetSettings },
}
