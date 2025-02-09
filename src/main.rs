#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use clap::{Parser, Subcommand};

#[cfg(all(not(feature = "client"), not(feature = "server")))]
compile_error!("at least one of `client` or `server` feature has to be enabled");

#[cfg(feature = "client")]
mod client;
mod protocol;
#[cfg(feature = "server")]
mod server;
mod shared;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    pub mode: Option<AppMode>,
}

#[derive(Subcommand, Debug, Clone, Default)]
pub enum AppMode {
    #[cfg(feature = "client")]
    #[default]
    Client,
    #[cfg(feature = "server")]
    #[cfg_attr(not(feature = "client"), default)]
    Server,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();

    #[cfg(feature = "client")]
    if matches!(cli.mode.clone().unwrap_or_default(), AppMode::Client) {
        app.add_plugins(client::ClientPlugin);
    }

    #[cfg(feature = "server")]
    if matches!(cli.mode.unwrap_or_default(), AppMode::Server) {
        app.add_plugins(server::ServerPlugin);
    }

    app.add_plugins(shared::SharedPlugin);

    app.run();
}
