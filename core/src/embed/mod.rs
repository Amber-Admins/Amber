pub mod engine;
pub mod registry;
pub mod storage;

pub use engine::{EmbedEngine, EmbedError};
pub use registry::{load_registry, OllamaDefaultConfig, Registry, TierConfig};
pub use storage::EmbeddingRow;
