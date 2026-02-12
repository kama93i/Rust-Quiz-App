//! Quiz server module.
//!
//! Provides WebSocket-based multiplayer quiz hosting.

mod commands;
mod server;
mod state;
mod ui;

pub use server::run;
