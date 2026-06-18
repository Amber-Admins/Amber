pub mod engine;
pub mod registry;

pub use engine::{EmbedEngine, EmbedError};
pub use registry::{load_registry, OllamaDefaultConfig, Registry, TierConfig};
