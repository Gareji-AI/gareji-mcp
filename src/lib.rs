//! Thin MCP translation layer for Gareji Core and Board Interfaces.

mod backend;
mod server;
mod types;

pub use backend::{FixtureBackend, GarejiBackend};
pub use server::GarejiMcpServer;
