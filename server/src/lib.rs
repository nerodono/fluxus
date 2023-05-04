/// # Protocols that used to manage proxies
pub mod protocols;

/// # Declarative macros
pub mod decl;

/// # Errors that can occur during serving
pub mod error;

/// # Server configuration
pub mod config;

/// # Useful helping stuff
pub mod utils;

/// # Data structures used in the servers
pub mod data;

/// # Slaves that will be manipulated by the master protocol
pub mod slaves;

/// # Structures that used to manipulate slave servers
pub mod servers;

/// # Dispatcher & specific handlers for the slave servers
pub mod communication;

/// # Functionality related to the HTTP server
#[cfg(feature = "http")]
pub mod http;
