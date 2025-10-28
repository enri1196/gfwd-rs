#[derive(Debug)]
pub enum SourceDialogRequest {
    SetSource(String),
    ValidateSource,
    Add,
    Cancel,
}

#[derive(Debug)]
pub enum SourceDialogResponse {
    SourceAdded { address: String },
}
