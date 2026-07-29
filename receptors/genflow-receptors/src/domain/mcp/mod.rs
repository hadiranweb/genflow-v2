//! MCP Domain Submodule — Core MCP types (pure domain, no runtime)
//!
//! Runtime traits (McpRepository, McpCache, McpBuilder) live in the
//! mcp-registry island. Here we only define domain types and the
//! non-async error type.

pub mod mcp_builder;
pub mod mcp_context;
pub mod mcp_error;

pub use mcp_builder::McpContextBuilder;
pub use mcp_context::{
    FragmentRole, McpBundle, McpContext, McpContextLink, McpLinkType, McpPromptFragment, McpScope,
    McpStatus, McpType, ResolutionMetadata,
};
pub use mcp_error::McpError;
