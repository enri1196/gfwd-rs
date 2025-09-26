mod app;
mod core;
mod messages;
mod models;
mod ui;
mod utils;

use relm4::prelude::*;

use crate::app::App;
use crate::utils::constants::APP_ID;

fn main() {
    let app = RelmApp::new(APP_ID);
    app.run_async::<App>(());
}
