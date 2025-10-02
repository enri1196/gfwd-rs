use crate::models::IPSetSettings;

#[derive(Debug)]
pub enum IPSetViewRequest {
    LoadIPSets,
    UpdateIPSets(Vec<String>),
    ShowCreateDialog,
    CreateIPSet(IPSetSettings),
    DeleteIPSet(String),
    SelectIPSet(String),
}

#[derive(Debug)]
pub enum IPSetViewResponse {
    IPSetSelected(String),
    IPSetCreated(String),
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
    RemoveEntry(String),
    Create,
    Cancel,
}

#[derive(Debug)]
pub enum IPSetDialogResponse {
    IPSetCreated { settings: IPSetSettings },
}