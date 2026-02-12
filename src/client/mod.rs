//! Quiz client module.
//!
//! Provides WebSocket-based multiplayer quiz client.

mod client;
mod state;
mod ui;

pub use client::run;
