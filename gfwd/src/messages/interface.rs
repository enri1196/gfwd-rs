#[derive(Debug)]
pub enum InterfaceDialogRequest {
    SetInterface(String),
    SelectInterface(u32), // Index of selected interface
    ValidateInterface,
    Add,
    Cancel,
}

#[derive(Debug)]
pub enum InterfaceDialogResponse {
    InterfaceAdded { name: String },
}