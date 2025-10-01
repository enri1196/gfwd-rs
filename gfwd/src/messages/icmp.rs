#[derive(Debug)]
pub enum IcmpDialogRequest {
    SetSelectedIcmp(String),
    Add,
    Cancel,
}

#[derive(Debug)]
pub enum IcmpDialogResponse {
    IcmpSelected { name: String },
}