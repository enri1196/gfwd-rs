#[derive(Debug)]
pub enum InterfaceDialogRequest {
    SetInterface(String),
    ValidateInterface,
    Add,
    Cancel,
}

#[derive(Debug)]
pub enum InterfaceDialogResponse {
    InterfaceAdded { name: String },
}