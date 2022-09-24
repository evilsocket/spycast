#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use clap::Parser;

mod mdns;

#[cfg(not(feature = "ui"))]
mod display;

#[cfg(feature = "ui")]
mod ui;
#[cfg(feature = "ui")]
use std::thread;

use mdns::discovery::{Agent, MappedEndpoints, SharedEndpoints};

#[derive(Parser, Default, Debug, Clone)]
struct Arguments {
    /// When in active mode send mDNS queries at this interval.
    #[clap(long, default_value_t = 5)]
    query_interval: u64,
    /// Only show results from this address.
    #[clap(long)]
    address: Option<String>,
    /// Do not execute queries, listen only.
    #[clap(long)]
    passive: bool,
    /// Save discovered endpoints as JSON files inside this folder.
    #[clap(long)]
    save_path: Option<String>,
}

fn save_to_path(path: &String, endpoints: &MappedEndpoints) {
    let path = std::path::Path::new(path);

    std::fs::create_dir_all(&path).unwrap();

    for endpoint in endpoints.values() {
        let filepath = path.join(format!("{}.json", endpoint.address));
        let json = serde_json::to_string_pretty(&endpoint).unwrap();
        std::fs::write(filepath, &json).unwrap();
    }
}

#[cfg(feature = "ui")]
fn start(args: Arguments) {
    // create a shared object for the agent and the UI
    let shared = Arc::new(Mutex::new(HashMap::new()));
    // create the agent
    let mut agent = Agent::new(args.query_interval, args.passive, args.address).unwrap();

    let state = shared.clone();
    // start the agent on its own thread
    thread::spawn(move || {
        agent.start(|endpoints: SharedEndpoints| {
            if let Ok(mut guard) = shared.lock() {
                if let Ok(epoints) = endpoints.lock() {
                    // update the UI state
                    _ = std::mem::replace(&mut *guard, epoints.clone());
                    // save to disk
                    if let Some(path) = &args.save_path {
                        save_to_path(path, &*epoints);
                    }
                }
            }
        });
    });

    // show the UI
    ui::run(state);
}

#[cfg(not(feature = "ui"))]
fn start(args: Arguments) {
    // create the agent
    let mut agent = Agent::new(args.query_interval, args.passive, args.address).unwrap();

    agent.start(|endpoints: SharedEndpoints| {
        if let Ok(guard) = endpoints.lock() {
            clearscreen::clear().unwrap();
            for endpoint in (*guard).values() {
                // display
                display::endpoint(endpoint);
            }

            // save to disk
            if let Some(path) = &args.save_path {
                save_to_path(path, &*guard);
            }
        }
    });
}

fn main() -> Result<(), String> {
    let args = Arguments::parse();

    start(args);

    Ok(())
}
