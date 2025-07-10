pub mod agent_builder;
pub mod macros;
pub mod models;

// Re-export the hash_map_fn! macro so {"key" => "value"} syntax works
pub use sugars_macros::hash_map_fn;

// Re-export the FluentAi builder and all required types
pub use agent_builder::{
    exec_to_text, Agent, AgentRoleBuilder, Context, Directory, File, Files, FluentAi, Github,
    Library, MessageChunk, MessageRole, NamedTool, Perplexity, Stdio, Tool,
};

// Re-export models for convenient access
pub use models::*;

// The hash_map_fn! macro is pushed down INTO the builder
// Users don't see it - they just use {"key" => "value"} syntax
