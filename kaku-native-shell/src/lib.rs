pub mod actions;
pub mod app_controller;
pub mod runtime_bridge;
pub mod snapshot;
pub mod styles;
pub mod terminal;
pub mod view;

use adw::prelude::*;
use app_controller::AppController;
use gtk::gio;
use std::fs;
use std::path::PathBuf;
use styles::install_css;

pub fn run() {
    install_bundled_fontconfig();
    if let Err(err) = adw::init() {
        eprintln!("failed to initialize GTK/Adwaita: {err:#}");
        return;
    }
    install_promise_scheduler();
    install_css();
    let app = adw::Application::builder()
        .application_id("local.shinchan.native-shell")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.set_register_session(false);
    app.connect_activate(|app| {
        if let Some(window) = app.active_window() {
            window.show();
            return;
        }

        let controller = AppController::new(app);
        controller.rerender();
        controller.window.show();
        glib::timeout_add_local_once(std::time::Duration::from_millis(120), {
            let controller = controller.clone();
            move || {
                controller.bootstrap();
                controller.maybe_run_smoke_harness();
            }
        });
    });
    app.run();
}

fn install_promise_scheduler() {
    if promise::spawn::is_scheduler_configured() {
        return;
    }

    promise::spawn::set_schedulers(
        Box::new(|runnable| {
            glib::MainContext::default().invoke(move || {
                runnable.run();
            });
        }),
        Box::new(|runnable| {
            glib::MainContext::default().invoke(move || {
                runnable.run();
            });
        }),
    );
}

fn install_bundled_fontconfig() {
    let font_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    if !font_dir.is_dir() {
        return;
    }

    let runtime_root = std::env::temp_dir().join("shin-chan-fontconfig");
    let cache_dir = runtime_root.join("cache");
    if fs::create_dir_all(&cache_dir).is_err() {
        return;
    }

    let fonts_conf = runtime_root.join("fonts.conf");
    let escaped_font_dir = font_dir.to_string_lossy().replace('&', "&amp;");
    let escaped_cache_dir = cache_dir.to_string_lossy().replace('&', "&amp;");
    let config = format!(
        r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
  <include ignore_missing="yes">/etc/fonts/fonts.conf</include>
  <include ignore_missing="yes">/usr/local/etc/fonts/fonts.conf</include>
  <include ignore_missing="yes">/opt/homebrew/etc/fonts/fonts.conf</include>
  <dir>{escaped_font_dir}</dir>
  <cachedir>{escaped_cache_dir}</cachedir>
</fontconfig>
"#
    );

    if fs::write(&fonts_conf, config).is_err() {
        return;
    }

    std::env::set_var("FONTCONFIG_FILE", &fonts_conf);
    std::env::set_var("XDG_CACHE_HOME", &runtime_root);
}
