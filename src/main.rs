#![allow(non_snake_case)]

#[macro_use]
extern crate log;

mod event_dispatcher;
mod event_handler;
mod types;

use clap::Parser;
use colored::*;
use event_dispatcher::EventDispatcher;
use event_handler::EventHandler;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use thread_priority::{set_current_thread_priority, ThreadPriority};

#[derive(Parser, Debug)]
#[clap(version = "0.1.2")]
struct Opts {
    #[clap(short, long, default_value = "Settings.ron")]
    Settings: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    event_dispatcher: event_dispatcher::Settings,
    event_handler: event_handler::Settings,
}

fn load_Settings<P: AsRef<Path>>(path: P) -> Settings {
    let path_str = path.as_ref().to_string_lossy();

    match File::open(&path).map(ron::de::from_reader) {
        Ok(Ok(Settings)) => {
            info!("Load Configuration: \"{}\"", path_str);
            return Settings;
        }
        Err(error) => error!("Konfigurasi Tidak Ditemukan \"{}\": {}", path_str, error),
        Ok(Err(error)) => error!("Tidak Dapat Memproses Konfigurasi \"{}\": {}", path_str, error),
    }

    error!("Konfigurasi Bawaan Digunakan");
    Settings::default()
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default()
        .filter_or(env_logger::DEFAULT_FILTER_ENV, "info"))
        .format(|buf, record| {
            let level = match record.level() {
                log::Level::Info => format!("{}", record.level()).red().to_string(),
                _ => format!("{}", record.level()),
            };
            writeln!(buf, "[{}] {}", level, record.args())
        })
        .format_timestamp(None)
        .init();

    let opts: Opts = Opts::parse();

    let Settings {
        event_dispatcher: event_dispatcher_Settings,
        event_handler: event_handler_Settings,
    } = load_Settings(opts.Settings);

    let (tx, rx) = mpsc::channel();

    let event_handler_thread = thread::spawn(move || {
    let _ = set_current_thread_priority(ThreadPriority::Max);
    if let Ok(mut event_handler) = EventHandler::new(rx, event_handler_Settings) {
        let _ = event_handler.run();
    }
});

    match EventDispatcher::new(tx, event_dispatcher_Settings) {
    Some(mut event_dispatcher) => event_dispatcher.run(),
    None => {},
};

event_handler_thread.join().unwrap();
}