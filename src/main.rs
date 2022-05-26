#[macro_use]
extern crate dotenv_codegen;

mod config;
mod consts;
mod crab_row;
mod crab_tabs;
mod music_object;
mod utils;
mod window;
mod daemon;

use gtk::gdk::Display;
use gtk::prelude::*;
use gtk::Application;
use gtk::{gio, CssProvider, StyleContext};
use std::fs::File;
use std::io::Write;
use std::process::exit;
use gtk::glib::{clone, MainContext, PRIORITY_DEFAULT, Receiver};

use crate::config::Config;
use crate::utils::{display_err};
use crate::daemon::{CrabDaemonClient, CrabDaemonServer};
use consts::*;
use window::Window;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    let arg = args.nth(1);

    if let Some(arg) = arg {
        match arg.as_str() {
            "--generate-config" => {
                let mut file = File::create(format!(
                    "{}{}",
                    dirs::config_dir().unwrap().as_os_str().to_str().unwrap(),
                    CONFIG_DEFAULT_PATH
                ))
                .unwrap();

                file.write_all(CONFIG_DEFAULT_STRING.as_bytes()).unwrap();

                println!("{}", CONFIG_GENERATED);
                exit(0);
            }
            "--daemon" => {
                let crab_daemon = CrabDaemonServer::new();

                let app = Application::builder().application_id(APP_ID).build();

                app.connect_startup(|_| load_css());
                app.connect_activate(|app| build_ui(app, false));

                app.run();

                crab_daemon.start();
            },
            "--show" => {
                let crab_daemon = CrabDaemonClient::new();
                crab_daemon.run_method("ShowWindow");
            }
            a => display_err(format!("Uknown parameter: {}", a).as_str()),
        }
    }

    gio::resources_register_include!("crab-launcher.gresource").expect(ERROR_RESOURCES);

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| load_css());
    app.connect_activate(|app| build_ui(app, true));

    app.run();
}

fn load_css() {
    let provider = CssProvider::new();

    Config::new().apply(&provider);

    StyleContext::add_provider_for_display(
        &Display::default().expect(ERROR_DISPLAY),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application, show_window: bool) {
    let window = Window::new(app);

    let (_tx, rx) = MainContext::channel::<bool>(PRIORITY_DEFAULT);

    rx.attach(None, clone!(@strong window => move |show_window| {
        println!("Got the event!");

        if show_window {
            window.present();
        }
        else {
            window.hide();
        }

        Continue(true)
    }));

    if show_window {
        window.present();
    }
}
