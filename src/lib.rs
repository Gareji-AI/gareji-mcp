//! Thin MCP translation layer for Gareji Core and Board Interfaces.

mod backend;
mod core_backend;
mod server;
mod types;

pub use backend::{FixtureBackend, GarejiBackend};
pub use core_backend::CoreBackend;
pub use server::GarejiMcpServer;
