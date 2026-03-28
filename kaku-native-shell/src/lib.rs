pub mod app_controller;
pub mod actions;
pub mod runtime_bridge;
pub mod snapshot;

use adw::prelude::*;
use app_controller::{install_css, AppController};

pub const APP_ID: &str = "fun.tw93.kaku.NativeShell";

pub fn run() {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| install_css());
    app.connect_activate(|app| {
        let controller = AppController::new(app);
        controller.bootstrap();
        controller.window.present();
    });
    app.run();
}
