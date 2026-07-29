//! genflow-mcp-registry — MCP Registry Island
//!
//! Ultra-fine-grained MCP: each MCP Type as an independent Cell.
//! Runtime services: Repository, Cache, Builder, Resolver.

pub mod runtime;
pub mod traits;

pub use runtime::builder::McpBuilderImpl;
pub use runtime::cache::RedisMcpCache;
pub use runtime::repository::PgMcpRepository;
pub use runtime::resolver::McpResolver;

pub use traits::{McpBuilder, McpCache, McpRepository, McpRuntimeError};
