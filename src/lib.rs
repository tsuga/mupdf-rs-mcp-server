//! MuPDF MCP Server library.
//!
//! This library provides an MCP server that exposes PDF reading and
//! manipulation capabilities using MuPDF.

pub mod error;
pub mod server;
pub mod state;
pub mod tools;

pub use error::{MupdfServerError, Result};
pub use server::MupdfServer;
pub use state::DocumentStore;
